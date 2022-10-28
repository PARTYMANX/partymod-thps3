cmake_minimum_required(VERSION 3.10)

project(thps3patch)

set(SRC "${CMAKE_SOURCE_DIR}/src")
set(THIRDPARTY "${CMAKE_SOURCE_DIR}/thirdparty")

set(SOURCE_FILES
	"${SRC}/main.c"
	"${SRC}/patch.c"
	"${SRC}/config.c"
	"${SRC}/input.c"
	"${SRC}/hash.c"
	"${SRC}/script.c"
	"${SRC}/patches.c"
)

set(HEADER_FILES
	"${SRC}/global.h"
	"${SRC}/patch.h"
	"${SRC}/config.h"
	"${SRC}/input.h"
	"${SRC}/hash.h"
	"${SRC}/script.h"
)

list(APPEND INCLUDES ${THIRDPARTY})

# add sdl2
find_package(SDL2 REQUIRED)
list(APPEND LIBS 
	SDL2::SDL2
	SDL2::SDL2main
)
list(APPEND INCLUDES ${SDL2_INCLUDE_DIRS})

add_library(partymod SHARED)

target_sources(partymod PRIVATE
	${SOURCE_FILES}
	${HEADER_FILES}
)

target_link_libraries(partymod PUBLIC ${LIBS})

target_include_directories(partymod PUBLIC
	"${PROJECT_SOURCE_DIR}/src"
	${INCLUDES}
)

add_executable(partyconfig)

target_sources(partyconfig PUBLIC config/main.c)
target_link_libraries(partyconfig PRIVATE 
	${LIBS}
	comctl32.lib
	UxTheme.lib
)
target_include_directories(partyconfig PUBLIC
	"${PROJECT_SOURCE_DIR}/src"
	${INCLUDES}
)

add_executable(incbin)

target_sources(incbin PUBLIC 
	"${THIRDPARTY}/incbin/incbin.c"
	"${THIRDPARTY}/incbin/incbin.h"
)

add_executable(partypatcher)

target_sources(partypatcher PUBLIC
	"${CMAKE_SOURCE_DIR}/patcher/main.c"
	"${CMAKE_SOURCE_DIR}/patcher/data.c"
)

target_include_directories(partypatcher PUBLIC
	"${CMAKE_SOURCE_DIR}/src"
	${THIRDPARTY}
)