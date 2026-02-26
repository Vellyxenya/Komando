# Komando
A command line utility to help you organize and easily access your commands.

# Install from `Crates.io`
Run:
```bash
cargo install komando
```

Then source your `.bashrc`:
```bash
source ~/.bashrc
```

Verify the installation works by running:
```bash
komando --help
```

# Build from source
Run:
```bash
cargo build --release
sudo cp target/release/komando_exec /usr/local/bin/
```

And don't forget to source your `.bashrc`:
```bash
source ~/.bashrc
```

Then verify the installation by running:
```bash
komando --help
```

# Usage

## Saving a command
Run:
```bash
komando --save
```
This will save the previous command entered in the terminal, along with the directory at which it was run.

**Note:** Komando automatically detects duplicate commands in the same directory and will warn you if you try to save the same command twice.

## Listing all saved commands
Run:
```bash
komando --list
```
This will display all your saved commands with their directories and unique IDs.

## Searching and executing a command
Run:
```bash
komando --query <YOUR_QUERY>
```
This will search your saved commands for any command that contains the given query and list them. You will then be presented with an interactive terminal where you can choose the command using the `UP` and `DOWN` arrows.

After choosing the command with `ENTER`, you can still edit it. At this point, hitting `ENTER` will execute the selected command.

**Note:** Each time you execute a command through komando, it tracks usage statistics for potential future features.

## Deleting a command
Run:
```bash
komando --delete <COMMAND_ID>
```
This will delete the command with the specified ID. You can find command IDs using `komando --list`.

# License
This project is licensed under the MIT License.