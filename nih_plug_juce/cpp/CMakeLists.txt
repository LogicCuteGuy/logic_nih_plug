# CMakeLists.txt for JUCE FFI Bridge
#
# This CMake configuration builds JUCE modules and the FFI bridge library.
# It is invoked from the Rust build.rs script.

cmake_minimum_required(VERSION 3.15)
project(nih_plug_juce_bridge LANGUAGES C CXX)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Ensure JUCE_SOURCE_DIR is set
if(NOT DEFINED JUCE_SOURCE_DIR)
    message(FATAL_ERROR "JUCE_SOURCE_DIR must be defined")
endif()

# Add JUCE subdirectory
# We set JUCE_MODULES_ONLY to avoid building examples and extras
set(JUCE_MODULES_ONLY ON CACHE BOOL "Only configure JUCE modules" FORCE)
set(JUCE_BUILD_EXAMPLES OFF CACHE BOOL "Don't build JUCE examples" FORCE)
set(JUCE_BUILD_EXTRAS OFF CACHE BOOL "Don't build JUCE extras" FORCE)

add_subdirectory(${JUCE_SOURCE_DIR} juce EXCLUDE_FROM_ALL)

# Platform detection
if(APPLE)
    set(JUCE_PLATFORM_MACOS TRUE)
    add_compile_definitions(JUCE_MAC=1)
elseif(WIN32)
    set(JUCE_PLATFORM_WINDOWS TRUE)
    add_compile_definitions(JUCE_WINDOWS=1)
elseif(UNIX)
    set(JUCE_PLATFORM_LINUX TRUE)
    add_compile_definitions(JUCE_LINUX=1)
endif()

# Create the bridge library
# For now, this is a minimal library that will be expanded in later tasks
add_library(nih_plug_juce_bridge STATIC
    juce_bridge.cpp
    component_bridge.cpp
    graphics_bridge.cpp
    widget_bridge.cpp
    timer_bridge.cpp
    colour_bridge.cpp
    font_bridge.cpp
    image_bridge.cpp
    path_bridge.cpp
    flexbox_bridge.cpp
    document_window_bridge.cpp
    tabbed_component_bridge.cpp
    list_box_bridge.cpp
    tree_view_bridge.cpp
    alert_window_bridge.cpp
    file_chooser_bridge.cpp
    drawable_bridge.cpp
    lookandfeel_bridge.cpp
    parameter_attachment_bridge.cpp
    message_thread_bridge.cpp
)

# Link JUCE modules
# These are the core modules needed for GUI functionality
target_link_libraries(nih_plug_juce_bridge
    PUBLIC
        juce::juce_core
        juce::juce_events
        juce::juce_graphics
        juce::juce_gui_basics
)

# Set include directories
target_include_directories(nih_plug_juce_bridge
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${JUCE_SOURCE_DIR}/modules
)

# Platform-specific compiler flags
if(JUCE_PLATFORM_MACOS)
    target_compile_options(nih_plug_juce_bridge PRIVATE
        -Wall
        -Wextra
        -Wno-unused-parameter
    )
    # Use libc++ on macOS
    target_compile_options(nih_plug_juce_bridge PUBLIC -stdlib=libc++)
    target_link_options(nih_plug_juce_bridge PUBLIC -stdlib=libc++)
    
elseif(JUCE_PLATFORM_LINUX)
    target_compile_options(nih_plug_juce_bridge PRIVATE
        -Wall
        -Wextra
        -Wno-unused-parameter
    )
    # Enable position-independent code for static library
    set_target_properties(nih_plug_juce_bridge PROPERTIES
        POSITION_INDEPENDENT_CODE ON
    )
    
elseif(JUCE_PLATFORM_WINDOWS)
    target_compile_options(nih_plug_juce_bridge PRIVATE
        /W4
        /wd4100  # Unused parameter
        /wd4127  # Conditional expression is constant
    )
    # Disable some JUCE warnings on Windows
    target_compile_definitions(nih_plug_juce_bridge PRIVATE
        _CRT_SECURE_NO_WARNINGS
        _SILENCE_CXX17_CODECVT_HEADER_DEPRECATION_WARNING
    )
endif()

# Disable JUCE analytics and crash reporting
target_compile_definitions(nih_plug_juce_bridge PUBLIC
    JUCE_DISPLAY_SPLASH_SCREEN=0
    JUCE_REPORT_APP_USAGE=0
    JUCE_WEB_BROWSER=0
    JUCE_USE_CURL=0
)

# Set output directory
set_target_properties(nih_plug_juce_bridge PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)

# Print configuration info
message(STATUS "JUCE FFI Bridge Configuration:")
message(STATUS "  JUCE Source: ${JUCE_SOURCE_DIR}")
message(STATUS "  Platform: ${CMAKE_SYSTEM_NAME}")
message(STATUS "  C++ Standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  Build Type: ${CMAKE_BUILD_TYPE}")
