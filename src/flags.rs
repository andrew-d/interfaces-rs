bitflags! {
    /// Represents a set of flags that describe the state of an interface.  This corresponds to the
    /// flags that are returned from the `SIOCGIFFLAGS` syscall
    pub struct InterfaceFlags: u32 {
        /// Interface is up.
        const IFF_UP = 0x1;

        /// Broadcast address valid.
        const IFF_BROADCAST = 0x2;

        /// Turn on debugging.
        const IFF_DEBUG = 0x4;

        /// Is a loopback net.
        const IFF_LOOPBACK = 0x8;

        /// Interface is point-to-point link.
        const IFF_POINTOPOINT = 0x10;

        /// Avoid use of trailers.
        const IFF_NOTRAILERS = 0x20;

        /// Resources allocated.
        const IFF_RUNNING = 0x40;

        /// No address resolution protocol.
        const IFF_NOARP = 0x80;

        /// Receive all packets.
        const IFF_PROMISC = 0x100;

        /// Receive all multicast packets.
        const IFF_ALLMULTI = 0x200;

        /// Master of a load balancer.
        const IFF_MASTER = 0x400;

        /// Slave of a load balancer.
        const IFF_SLAVE = 0x800;

        /// Supports multicast.
        const IFF_MULTICAST = 0x1000;

        /// Can set media type.
        const IFF_PORTSEL = 0x2000;

        /// Auto media select active.
        const IFF_AUTOMEDIA = 0x4000;

        /// Dialup device with changing addresses.
        const IFF_DYNAMIC = 0x8000;
    }
}
