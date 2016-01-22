// All modern Unices include this file, which can make things a bit nicer.  Pull it in.
#if defined(__unix__) || (defined(__APPLE__) && defined(__MACH__))
    #include <sys/param.h>
#endif

// Figuring out which OS it is is a giant PITA in C :-(
#ifdef _WIN32
    #define OS_WINDOWS 1

    #ifdef _WIN64
        #define OS_WIN64 1
    #else
        #define OS_WIN32 1
    #endif
#elif __APPLE__
    #include "TargetConditionals.h"

    #if TARGET_IPHONE_SIMULATOR
        #define OS_IOS 1
        #define OS_IOS_SIMULATOR 1
    #elif TARGET_OS_IPHONE
        #define OS_IOS 1
    #elif TARGET_OS_MAC
        #define OS_MACOS 1
    #else
        #error "Unknown Apple platform"
    #endif
#elif __linux__
    #define OS_LINUX 1
#elif defined(BSD)
    #define OS_BSD 1
#endif

// General platforms
#if __unix__
    #define OS_UNIX 1
#endif

#if defined(_POSIX_VERSION)
    #define OS_POSIX 1
#endif

#if !defined(OS_LINUX) && !defined(OS_MACOS) && !defined(OS_IOS) && !defined(OS_WINDOWS) && !defined(OS_BSD)
    #error "Unknown compiler"
#endif


#include <stdlib.h>
#include <stdint.h>

#include <sys/ioctl.h>
#include <sys/socket.h>
#include <net/if.h>
#include <ifaddrs.h>


#if defined(OS_MACOS) || defined(OS_BSD)
#include <net/if_dl.h>
#endif


uint8_t* rust_LLADDR(struct ifaddrs* ifap) {
    return (uint8_t *)LLADDR((struct sockaddr_dl *)(ifap)->ifa_addr);
}
