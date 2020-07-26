cmake_minimum_required(VERSION 3.16)
project(notilus C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_FLAGS "-W -Wall -Wextra -Werror")

include(FindPkgConfig)

add_library(notilus STATIC src/notilus.c)

if (PkgConfig_FOUND)
    pkg_check_modules(LIBNOTIFY libnotify>=0.7)
endif ()

if (NOT LIBNOTIFY_LINK_LIBRARIES)
    message(FATAL_ERROR "libnotify not found")
endif ()

target_include_directories(notilus INTERFACE ${PROJECT_BINARY_DIR}/include)
target_link_libraries(notilus PRIVATE ${LIBNOTIFY_LINK_LIBRARIES})
target_include_directories(notilus PUBLIC ${LIBNOTIFY_INCLUDE_DIRS})

install(TARGETS notilus DESTINATION lib)
install(FILES include/notilus.h DESTINATION include)