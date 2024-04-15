# minasan
> _In the ninja world, those who don't follow the rules are trash._  
> _But, those who abandon their friends are even worse than trash._
> 
> _-_ Obito Uchiha, "Naruto Shippuden"

Telegram Bot to tag all the chat members

:warning: __The bot is NOT DONE yet.__ :warning:  

# Usage

Add [this young man](https://t.me/ryanbotling_bot) to your chat. The bot has 
no access to your messages, so it is completely safe.  
In order to start, type in `/minasanstart` in your telegram chat.   
This will create a poll, 
every group member wanting to be tagged should choose `I do` option.

## Commands

| Command           | Description                                                        |
|-------------------|--------------------------------------------------------------------|
| `/minasan`        | Tags all the chat members, consented to be tagged.                 |
| `/minasanstart`   | Starts the poll to record all consented chat members.              |
| `/minasanhelp`    | Prints commands description.                                       |
| `/minasanpoll`    | Resends the poll, if one was created.                              |
| `/minasankill`    | Deletes the poll and removes the bot from the chat.                |
| `/minasanrestart` | Restarts the poll, deleting the results of the previus active one. |

# How it works
The bot tracks poll answers of all chat members, remembering only 
the consented ones.   
One can exclude themselves from the list by just refraining from answering the poll, or   
by just selecting `I don't` option later.

# Self-Hosting
`minasan` is available as either `cargo crate` and `docker image`.   
You can just run
```commandline
cargo install minasan
minasan --path /path/to/storage --interval 3600
```
where `path` and `interval` correspond to path for storing collected user   
base and interval of its dump to disk in seconds.

One can also pull docker image  
```commandline
docker pull arseniybelkov/minasan
docker run ...
```