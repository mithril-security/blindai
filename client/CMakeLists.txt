# Build "_quote_verification" module: Python C++ extension to verify 
# the DCAP Attestation. Uses a patched version of Intel DCAP Quote Verification lib

cmake_minimum_required(VERSION 3.4...3.18)

project(_quote_verification)
link_directories(blindai/lib)
add_subdirectory(third_party/pybind11)
pybind11_add_module(_quote_verification blindai/cpp/wrapper.cc)

if(CMAKE_CXX_COMPILER_ID STREQUAL "Clang" OR CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    target_link_libraries(_quote_verification 
    PRIVATE verify
    PRIVATE QuoteVerification
    )
    
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "MSVC")
    add_definitions(-D_ITERATOR_DEBUG_LEVEL=0)
    target_link_libraries(_quote_verification 
        PRIVATE AppCore
        PRIVATE AttestationCommonsStatic
        PRIVATE QuoteVerification
        PRIVATE QuoteVerificationStatic
        PRIVATE Argtable3
    )
endif()

target_compile_features(_quote_verification PRIVATE cxx_std_17)
