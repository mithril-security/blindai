from operator import sub
import os
import setuptools
import subprocess
import platform
import re
import sys
from setuptools import Extension
from setuptools.command.build_ext import build_ext
from setuptools.command.build_py import build_py
import pkg_resources


# Platform check
SUPPORTED_PLATFORMS = {"LINUX", "WINDOWS", "DARWIN"}
PLATFORM = platform.system().upper()
if PLATFORM not in SUPPORTED_PLATFORMS:
    print("The platform : {}".format(platform.system().lower()))
    print("Building blindai client in your platform is not supported yet.")
    exit(1)

# Convert distutils Windows platform specifiers to CMake -A arguments
PLAT_TO_CMAKE = {
    "win32": "Win32",
    "win-amd64": "x64",
}

# AttestationLib Build Script
ATTESTATION_BUILD_SCRIPT = {
    "WINDOWS": {
        "build": [
            "powershell.exe",
            os.path.join(os.path.dirname(__file__), "scripts/buildAttestationLib.ps1"),
        ],
        "postbuild": [
            "powershell.exe",
            "Start-Process",
            "powershell.exe",
            "-Verb",
            "runAs",
            "-ArgumentList",
            "$"
            + '"{}"'.format(
                os.path.join(
                    os.path.dirname(__file__), "scripts/postBuildAttestationLib.ps1"
                )
            ),
        ],
    },
    "LINUX": {
        "build": [
            os.path.join(os.path.dirname(__file__), "scripts/buildAttestationLib.sh")
        ],
        "postbuild": [],
    },
    "DARWIN": {
        "build": [
            os.path.join(
                os.path.dirname(__file__), "scripts/buildAttestationLibDarwin.sh"
            )
        ],
        "postbuild": [],
    },
}

# Proto Files
PROTO_FILES = ["securedexchange.proto", "untrusted.proto", "proof_files.proto", "licensing.proto"]
PROTO_PATH = os.path.join(os.path.dirname(__file__), "proto")


# Util functions
def read(filename):
    return open(os.path.join(os.path.dirname(__file__), filename)).read()


def find_version():
    version_file = read("blindai/version.py")
    version_re = r"__version__ = \"(?P<version>.+)\""
    version = re.match(version_re, version_file).group("version")
    return version


def build_attestation_lib():
    subprocess.check_call(ATTESTATION_BUILD_SCRIPT[PLATFORM]["build"])
    if PLATFORM not in ["LINUX", "DARWIN"]:
        subprocess.check_call(ATTESTATION_BUILD_SCRIPT[PLATFORM]["postbuild"])


def generate_stub():
    import grpc_tools.protoc

    proto_include = pkg_resources.resource_filename("grpc_tools", "_proto")
    for file in PROTO_FILES:
        grpc_tools.protoc.main(
            [
                "grpc_tools.protoc",
                "-I{}".format(proto_include),
                "--proto_path={}".format(PROTO_PATH),
                "--python_out=blindai/pb",
                "--grpc_python_out=blindai/pb",
                "{}".format(file),
            ]
        )


def get_libs():
    if PLATFORM == "LINUX":
        return "lib/*.so"
    if PLATFORM == "DARWIN":
        return "lib/*.dylib"
    if PLATFORM == "WINDOWS":
        return "lib/*.dll"


# Build Classes
class CMakeExtension(Extension):
    def __init__(self, name, sourcedir=""):
        Extension.__init__(self, name, sources=[])
        self.sourcedir = os.path.abspath(sourcedir)


class CMakeBuild(build_ext):
    """Build the pybind11 module and add it as an extension to the package"""

    def build_extension(self, ext):
        extdir = os.path.abspath(os.path.dirname(self.get_ext_fullpath(ext.name)))

        if not extdir.endswith(os.path.sep):
            extdir += os.path.sep

        debug = int(os.environ.get("DEBUG", 0)) if self.debug is None else self.debug
        cfg = "Debug" if debug else "Release"

        cmake_generator = os.environ.get("CMAKE_GENERATOR", "")

        cmake_args = [
            f"-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={extdir}",
            f"-DPYTHON_EXECUTABLE={sys.executable}",
            f"-DCMAKE_BUILD_TYPE={cfg}",
        ]
        build_args = []
        if "CMAKE_ARGS" in os.environ:
            cmake_args += [item for item in os.environ["CMAKE_ARGS"].split(" ") if item]

        if self.compiler.compiler_type == "msvc":
            single_config = any(x in cmake_generator for x in {"NMake", "Ninja"})
            if not single_config:
                cmake_args += ["-A", PLAT_TO_CMAKE[self.plat_name]]

            if not single_config:
                cmake_args += [
                    f"-DCMAKE_LIBRARY_OUTPUT_DIRECTORY_{cfg.upper()}={extdir}"
                ]
                build_args += ["--config", cfg]

        if not os.path.exists(self.build_temp):
            os.makedirs(self.build_temp)

        subprocess.check_call(
            ["cmake", ext.sourcedir] + cmake_args, cwd=self.build_temp
        )
        subprocess.check_call(
            ["cmake", "--build", "."] + build_args, cwd=self.build_temp
        )
        if PLATFORM == "LINUX":
            # Change the run path for the _quote_verification.so
            edit_path = os.path.join(
                os.path.dirname(__file__), "scripts/postExtensionBuild.sh"
            )
            subprocess.check_call([edit_path])

        if PLATFORM == "DARWIN":
            # Change the run path for the _quote_verification.dylib
            edit_path = os.path.join(
                os.path.dirname(__file__), "scripts/postExtensionBuildDarwin.sh"
            )
            subprocess.check_call([edit_path])


class BuildPackage(build_py):
    def run(self):
        build_attestation_lib()
        generate_stub()
        super(BuildPackage, self).run()


setuptools.setup(
    name="blindai",
    author="Mithril-Security",
    version=find_version(),
    author_email="contact@mithrilsecurity.io",
    description="Client SDK for BlindAI Confidential Inference Server",
    license="Apache-2.0",
    long_description=read("README.md"),
    long_description_content_type="text/markdown",
    keywords="confidential computing inference client enclave sgx machine learning",
    url="https://www.mithrilsecurity.io/",
    packages=setuptools.find_packages(exclude=["blindai/cpp/wrapper.cc"]),
    package_data={"": [get_libs(), "tls/*.pem"]},
    ext_modules=[CMakeExtension("_quote_verification")],
    cmdclass={"build_ext": CMakeBuild, "build_py": BuildPackage},
    zip_safe=False,
    python_requires=">=3.6.8",
    install_requires=[
        "cryptography>=35.0.0",
        "toml",
        "grpcio==1.44",
        "grpcio-tools==1.44",
        "bitstring",
    ],
    extras_require={
        "dev": [
            "pybind11",
            "setuptools",
            "wheel",
            "check-wheel-contents",
            "auditwheel",
            "grpcio-tools==1.44",
            "grpcio==1.44",
        ]
    },
    classifiers=[
        "Programming Language :: Python :: 3",
        "Programming Language :: C++",
        "Operating System :: Unix",
        "Operating System :: Microsoft :: Windows",
        "Operating System :: MacOS :: MacOS X",
    ],
)
