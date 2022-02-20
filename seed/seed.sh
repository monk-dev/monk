cargo build -p monk-cli

rm monk.sqlite
rm -r ./downloads
rm -r ./index

./target/debug/monk add "As We May Think" seed/data/monolith_as_we_may_think.html "essentially monk but in the 1940's"
./target/debug/monk add "Learning Key-Value Store Design" seed/data/learned_key_value_stores.pdf "Interesting article on dynamic key-value storage designs"
./target/debug/monk add "Accelerating networking with AF_XDP" seed/data/monolith_af_xdp.html "Super fast packet capturing with AF\_XDP"
./target/debug/monk add "Multicast Listener Discovery (MLD) for IPv6" seed/data/monolith_mldv1_rfc.html "RFC that defines Mldv1"

echo
echo

./target/debug/monk list