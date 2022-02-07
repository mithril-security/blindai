cmake_minimum_required(VERSION 3.4...3.18)
project(pybind11_module)

link_directories(blindai/lib)

add_subdirectory(third_party/pybind11)

pybind11_add_module(pybind11_module blindai/cpp/wrapper.cc)
target_link_libraries(pybind11_module PRIVATE verify)
target_link_libraries(pybind11_module PRIVATE QuoteVerification)

target_compile_features(pybind11_module PRIVATE cxx_std_17)

