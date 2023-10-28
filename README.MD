###  BoltCard Command Control Bot
 - Control your physical BoltCard running on [BoltCard Service](https://github.com/boltcard/boltcard) via [Nostr](https://nostr.how/en/what-is-nostr) account
 - Command bot written in Rust for making updates to the card settings on the BoltCard Service.

Pre-requisites:
- Card Name - as is in the BoltCard Service DB (card_db) 'cards' table - 'card_name' column
- Nostr account private key

Running the bot:
 - BoltCard Service is running on the same host as the Bot
 - [Install Rust](https://www.rust-lang.org/tools/install)
 - clone this repo: *git clone https://github.com/ponthief/nostr_boltcard_bot.git*
 - edit/save __src/main.rs__: set **BECH32_SK** variable to your Nostr account private key (nsec)
 - compile binary in the top folder: __cargo build__
 - run BoltCard Bot: __cargo run__

To view commands supported enter: __/help__ in the private chat with Bot of your Nostr client app (Amethyst, IRIS etc.)

<img src= "https://github.com/ponthief/nostroBolt/blob/master/nostr_boltcard_bot_help.jpg" width="350" height="550">
