c2l
===

Create a `launchd` config file from a command.

# use

```
$ ./target/release/c2l com.ckampfe.myserver "PORT=5000 /Users/clark/code/myserver.sh start" --keepalive --run-at-load
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
        <dict>
                <key>Label</key>
                <string>com.ckampfe.myserver</string>
                <key>ProgramArguments</key>
                <array>
                        <string>/Users/clark/code/myserver.sh</string>
                        <string>start</string>
                </array>
                <key>EnvironmentVariables</key>
                <dict>
                        <key>PORT</key>
                        <string>5000</string>
                </dict>
                <key>RunAtLoad</key>
                <true />
                <key>KeepAlive</key>
                <true />
        </dict>
</plist>
```

Then, write this file to ~/Library/LaunchAgents

# install

```
$ git clone git@github.com:ckampfe/c2l.git
$ cd c2l
$ cargo install --path .
```
