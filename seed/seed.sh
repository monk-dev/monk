cargo build --release -p monk-cli

rm -r test-install

./target/release/monk add "As We May Think" seed/data/monolith_as_we_may_think.html "essentially monk but in the 1940's" -t monk old
./target/release/monk add "Learning Key-Value Store Design" seed/data/learned_key_value_stores.pdf "Interesting article on dynamic key-value storage designs" -t key-value datastructures
./target/release/monk add "Accelerating networking with AF_XDP" seed/data/monolith_af_xdp.html "Super fast packet capturing with AF\_XDP" -t kernel networking
./target/release/monk add "Multicast Listener Discovery (MLD) for IPv6" seed/data/monolith_mldv1_rfc.html "RFC that defines Mldv1" -t networking

echo
echo

./target/release/monk list