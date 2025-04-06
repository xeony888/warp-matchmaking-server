# Knockout Headless Server Unity Build

This is a headless server build for the game **Knockout** using Unity. It is designed to run in a command-line environment and accepts specific command-line arguments for configuration.

## Command-Line Arguments

The server supports the following command-line arguments:

| Argument       | Required | Default | Description |
|---------------|----------|---------|-------------|
| `-port`       | No       | 7777    | The port on which the server will listen. |
| `-username1`  | Yes      | N/A     | Username of the first player (display purposes only). |
| `-username2`  | Yes      | N/A     | Username of the second player (display purposes only). |
| `-player1token` | Yes    | N/A     | Token used to authenticate player 1. |
| `-player2token` | Yes    | N/A     | Token used to authenticate player 2. |

## Exit Codes

The server exits with specific codes based on the game's outcome:

- **1001** - Player 1 wins.
- **1002** - Player 2 wins.
Any other exit code means something went wrong.

## Usage Example

```sh
./KnockoutServer -port 9000 -username1 "PlayerOne" -username2 "PlayerTwo" -player1token "token123" -player2token "token456"
```

This command starts the Knockout server on port `9000` with specified player usernames and authentication tokens.

### Error fixes

sudo chmod +x ./path/to/executable