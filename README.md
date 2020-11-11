# Torrenter

The goal of this project is to have command line GUI torrent application. 
My inspiration for the project is the idea of having a simple torrent 
application that doesn't spam you with ads and remains in the terminal. 
As torrents are something I use all the time, I thought it would be cool 
to understand how the protocol works at the lowest level and learn Rust along the way.

There are a bunch of things that can be optimised and need improving. 
If you happen to see anything that could be done better in terms of design 
or simply Rust code I'd love to hear from you. 

## Things that need to be done

- [x] Get downloads working with multiple peers and concurrency.
- [x] Add the ability to download multiple files in a torrent.
- [ ] Check the hash of each piece before writing to the file.
- [ ] Add the ability to pause downloads and save download state.
- [ ] Reorganise the code for a more OOP approach.
- [ ] Add tests
- [ ] Setup a listener for seeding to peers. 
- [ ] Improve error handling and add reties for the tracker.
- [ ] Update tracker regularly and update list of peers.
- [ ] Add NAT traversal to access peers behind NAT.
- [ ] GUI for the terminal.


 
