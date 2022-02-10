import os
import setuptools
import subprocess
import platform
import re
import sys
import grpc_tools.protoc
from setuptools import Extension
from setuptools.command.build_ext import build_ext
from setuptools.command.build_py import build_py

proto_files = ["securedexchange.proto", "untrusted.proto"]

def read(filename):
    return open(os.path.join(os.path.dirname(__file__), filename)).read()

def find_version():
    version_file = read("blindai/version.py")
    version_re = r"__version__ = \"(?P<version>.+)\""
    version = re.match(version_re, version_file).group("version")
    return version

class CMakeExtension(Extension):
    def __init__(self, name, sourcedir=""):
        Extension.__init__(self, name, sources=[])
        self.sourcedir = os.path.abspath(sourcedir)

class CMakeBuild(build_ext):
    def build_extension(self, ext):

        if platform.system() == "Windows":
            raise RuntimeError("Currently, the library can only be built on linux")

        extdir = os.path.abspath(os.path.dirname(self.get_ext_fullpath(ext.name)))

        if not extdir.endswith(os.path.sep):
            extdir += os.path.sep

        debug = int(os.environ.get("DEBUG", 0)) if self.debug is None else self.debug
        cfg = "Debug" if debug else "Release"

        cmake_args = [
            f"-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={extdir}",
            f"-DPYTHON_EXECUTABLE={sys.executable}",
            f"-DCMAKE_BUILD_TYPE={cfg}",
        ]

        build_args = []

        if "CMAKE_ARGS" in os.environ:
            cmake_args += [item for item in os.environ["CMAKE_ARGS"].split(" ") if item]

        if "CMAKE_BUILD_PARALLEL_LEVEL" not in os.environ:
            # self.parallel is a Python 3 only way to set parallel jobs by hand
            # using -j in the build_ext call, not supported by pip or PyPA-build.
            if hasattr(self, "parallel") and self.parallel:
                # CMake 3.12+ only.
                build_args += [f"-j{self.parallel}"]

        if not os.path.exists(self.build_temp):
            os.makedirs(self.build_temp)

        subprocess.check_call(
            ["cmake", ext.sourcedir] + cmake_args, cwd=self.build_temp
        )
        subprocess.check_call(
            ["cmake", "--build", "."] + build_args, cwd=self.build_temp
        )
        edit_path = os.path.join(os.path.dirname(__file__), 'scripts/edit_runpath.sh')
        subprocess.check_call([edit_path])

class BuildPy(build_py):
    def run(self):
        # Generate the stub
        dir_path = os.path.join(os.path.dirname(__file__))
        proto_path = os.path.join(dir_path, "proto")

        for file in proto_files:
            grpc_tools.protoc.main([
                'grpc_tools.protoc',
                '--proto_path={}'.format(proto_path),
                '--python_out=blindai',
                '--grpc_python_out=blindai',
                '{}'.format(file)
            ])
        # Build the AttestationLib
        build_script = os.path.join(os.path.dirname(__file__), 'scripts/build.sh')
        subprocess.check_call([build_script])
        super(BuildPy, self).run()

setuptools.setup(
    name="blindai",
    author="Mithril-Security",
    version=find_version(),
    author_email="contact@mithrilsecurity.io",
    description="A python library for creating gRPC clients for blindai inference server",
    license="",
    keywords="confidential computing inference client enclave",
    url="www.github.com/mithril-security/blindai/client",
    packages=setuptools.find_packages(exclude=["blindai/cpp/wrapper.cc"]),
    package_data={"": ["lib/*.so", "tls/*.pem"]},
    ext_modules=[CMakeExtension("pybind11_module")],
    cmdclass={
        "build_ext": CMakeBuild,
        "build_py": BuildPy,
    },

    zip_safe=False,
    python_requires=">=3.6.9",
    install_requires=[
        "cryptography>=35.0.0",
        "toml",
        "grpcio",
        "grpcio-tools",
        "bitstring",
        "cbor2"
    ],
    extras_require={
        'dev': [
            'pybind11',
            'setuptools',
            'wheel',
            'patchelf',
            'check-wheel-contents',
            'auditwheel',
            'grpcio-tools',
            'grpcio'
        ]
    },
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: ",
        "Operating System :: Linux",
    ],
)
