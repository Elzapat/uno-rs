## TODO
* change settings in game lobby
* random position for uno and counter uno buttons
* draw a card whenever the player wants
* draw card by pressing on deck

## BUGS
* Naia server crashes when restarting a game ? `thread 'main' panicked at 'called Option::unwrap() on a None value', /home/morgan/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/naia-server-0.10.0/src/server.rs:565:73`
* When two start game events occur at the same time: `thread 'main' panicked at 'cannot sample empty range', /home/morgan/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/rand-0.8.5/src/rng.rs:134:9`
* When restarting a game, the new game is buggy (one client doesn't have current color, cards don't work as expected)
* [FIXED] When restarting a game, client sometimes has extra ghost cards
