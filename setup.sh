set +x

# Create TAP device if it doesn't already exist
ip link show tap-rust >/dev/null 2>&1 || sudo ip tuntap add mode tap name tap-rust user $USER

# Bring up the TAP interface
sudo ip link set tap-rust up

# IPv4 config
sudo ip addr add 192.168.42.100/24 dev tap-rust
sudo iptables -t nat -A POSTROUTING -s 192.168.42.0/24 -j MASQUERADE
sudo sysctl -w net.ipv4.ip_forward=1

# IPv6 config
sudo ip -6 addr add fe80::100/64 dev tap-rust scope link
sudo ip -6 addr add fdaa::100/64 dev tap-rust
sudo ip -6 route add fdaa::/64 dev tap-rust
sudo ip6tables -t nat -A POSTROUTING -s fdaa::/64 -j MASQUERADE
sudo sysctl -w net.ipv6.conf.all.forwarding=1
