<div align="center">
  <h1> Tenere </h1>
  <img src="assets/logo.png" alt="A crab in the moroccan desert"></img>
  <h2> TUI interface for LLMs written in Rust </h2>
</div>

## 📸 Demo

![Demo](https://github.com/pythops/tenere/assets/57548585/b33ed59b-1d94-4bc5-8e61-73e63f41137e)

<br>

## 🪄 Features

- Syntax highlights
- Chat history
- Save chats to files
- Vim keybinding (most common ops)
- Copy text from/to clipboard (works only on the prompt)
- Multiple backends
- Automatically load the last saved chat into history

<br>

## 💎 Supported Backends

- [x] ChatGPT
- [x] llama.cpp
- [x] ollama

<br>

## 🚀 Installation

<a href="https://repology.org/project/tenere/versions">
    <img src="https://repology.org/badge/vertical-allrepos/tenere.svg" alt="Packaging status" align="left">
</a>

<br>
<br>
<br>
<br>
<br>

### 📥 Binary releases

You can download the pre-built binaries from the [release page](https://github.com/pythops/tenere/releases)

### 📦 crates.io

`tenere` can be installed from [crates.io](https://crates.io/crates/tenere)

```shell
cargo install tenere
```

### ❄️ NixOS / Nix

Tenere is available in nixpkgs and can be installed via configuration.nix:

```nix
environment.systemPackages = with pkgs; [
  tenere
];
```
For non-NixOS systems, install directly with:
```nix
nix-env -iA nixpkgs.tenere
```

### 📱 Mobile (nix-on-droid)

Tenere works on Android via nix-on-droid ([demo](https://github.com/user-attachments/assets/c06e5650-0b5d-4f0a-816d-a2c1bd88774a)).

To set up ([tutorial](https://www.youtube.com/watch?v=XiVz2UR9epE)):

1. Install nix-on-droid from F-Droid
2. Add tenere to your packages in ".config/nixpkgs/nix-on-droid.nix":
3. Run ``nix-on-droid switch``
4. Create your config at ".config/tenere/config.toml"

### 🍺 Homebrew

```
brew install tenere
```

### ⚒️ Build from source

To build from the source, you need [Rust](https://www.rust-lang.org/) compiler and
[Cargo package manager](https://doc.rust-lang.org/cargo/).

Once Rust and Cargo are installed, run the following command to build:

```shell
cargo build --release
```

This will produce an executable file at `target/release/tenere` that you can copy to a directory in your `$PATH`.

<br>

## ⚙️ Configuration

Tenere can be configured using a TOML configuration file. By default, the configuration file is located at:

- **Linux**: `$HOME/.config/tenere/config.toml` or `$XDG_CONFIG_HOME/tenere/config.toml`
- **Mac**: `$HOME/Library/Application Support/tenere/config.toml`
- **Windows**: `~/AppData/Roaming/tenere/config.toml`

### 🛠 Custom Configuration Path

You can optionally specify a custom path for the configuration file using the `-c` flag. This allows you to override the default configuration file location.

### Example Usage

```sh
# Use the default configuration path
tenere

# Specify a custom configuration path
tenere -c ~/path/to/custom/config.toml
```

### General settings

Here are the available general settings:

- `llm`: the llm model name. Possible values are:
  - `chatgpt`
  - `llamacpp`
  - `ollama`

```toml
llm  = "chatgpt"
```

### Key bindings

Tenere supports customizable key bindings.
You can modify some of the default key bindings by updating the `[key_bindings]` section in the configuration file.
Here is an example with the default key bindings

```toml
[key_bindings]
show_help = '?'
show_history = 'h'
new_chat = 'n'
```

ℹ️ Note

> To avoid overlapping with vim key bindings, you need to use `ctrl` + `key` except for help `?`.

## Chatgpt

To use `chatgpt` as the backend, you'll need to provide an API key for OpenAI. There are two ways to do this:

Set an environment variable with your API key:

```shell
export OPENAI_API_KEY="YOUTR KEY HERE"
```

Or

Include your API key in the configuration file:

```toml
[chatgpt]
openai_api_key = "Your API key here"
model = "model_name"
url = "Either OpenAI format or SGLang format"

[chatgpt]
openai_api_key = "Your API key here"
model = "gpt-3.5-turbo"
url = "https://api.openai.com/v1/chat/completions"

[file_path]
history_file_path = "history.md"
result_vault_path = "result_vault.md"
```

The default model is set to `gpt-3.5-turbo`. Check out the [OpenAI documentation](https://platform.openai.com/docs/models/gpt-3-5) for more info.

## llama.cpp

To use `llama.cpp` as the backend, you'll need to provide the url that points to the server :

```toml
[llamacpp]
url = "http://localhost:8080/v1/chat/completions"
```

If you configure the server with an api key, then you need to provide it as well:

Setting an environment variable :

```shell
export LLAMACPP_API_KEY="YOUTR KEY HERE"
```

Or

Include your API key in the configuration file:

```toml
[llamacpp]
url = "http://localhost:8080/v1/chat/completions"
api_key = "Your API Key here"
```

More infos about llama.cpp api [here](https://github.com/ggerganov/llama.cpp/blob/master/examples/server/README.md)

## Ollama

To use `ollama` as the backend, you'll need to provide the url that points to the server with the model name :

```toml
[ollama]
url = "http://localhost:11434/api/chat"
model = "Your model name here"
```

More infos about ollama api [here](https://github.com/ollama/ollama/blob/main/docs/api.md#generate-a-chat-completion)

<br>

## ⌨️ Key bindings

### Global

These are the default key bindings regardless of the focused block.

`ctrl + n`: Start a new chat and save the previous one in history and save it to `tenere.archive-i` file in [data directory](https://docs.rs/dirs-next/latest/dirs_next/fn.data_dir.html). 

`Tab`: Switch the focus.

`j` or `Down arrow key`: Scroll down

`k` or `Up arrow key`: Scroll up

`ctrl + h` : Show chat history. Press `Esc` to dismiss it.

`ctrl + t` : Stop the stream response

`q` or `ctrl + c`: Quit the app

`?`: Show the help pop-up. Press `Esc` to dismiss it

`w`: When view the chat, save the current chat to a file. When view the history, save the selected chat to a file. Config `result_vault_path` or `history_path` in the config file to change the default path accordingly.

### Prompt

There are 3 modes like vim: `Normal`, `Visual` and `Insert`.

#### Insert mode

`Esc`: to switch back to Normal mode.

`Enter`: to create a new line.

`Backspace`: to remove the previous character.

#### Normal mode

`Enter`: to submit the prompt

<br>

`h or Left`: Move the cursor backward by one char.

`j or Down`: Move the cursor down.

`k or Up`: Move the cursor up.

`l or Right`: Move the cursor forward by one char.

`w`: [masked and disabled unless recaped] Move the cursor right by one word.

`b`: Move the cursor backward by one word.

`0`: Move the cursor to the start of the line.

`$`: Move the cursor to the end of the line.

`G`: Go to the end.

`gg`: Go to the top.

<br>

`a`: Insert after the cursor.

`A`: Insert at the end of the line.

`i`: Insert before the cursor.

`I`: Insert at the beginning of the line.

`o`: Append a new line below the current line.

`O`: Append a new line above the current line.

<br>

`x`: Delete one char under to the cursor.

`dd`: Cut the current line

`D`: Delete the current line and

`dw`: Delete the word next to the cursor.

`db`: Delete the word on the left of the cursor.

`d0`: Delete from the cursor to the beginning of the line.

`d$`: Delete from the cursor to the end of the line.

<br>

`C`: Change to the end of the line.

`cc`: Change the current line.

`c0`: Change from the cursor to the beginning of the line.

`c$`: Change from the cursor to the end of the line.

`cw`: Change the next word.

`cb`: Change the word on the left of the cursor.

<br>

`u`: Undo

`p`: Paste

#### Visual mode

`v`: Switch to visual.

`y`: Yank the selected text

<br>

## ⚖️ License

GNU General Public License v3.0 or later


## Testing feature

- [x] LM-studio, SGlang, Transwarp Sophon-LLMOps and openai format are both supported in the name of `chatgpt`;
- [x] Chat history is saved in the data directory.

## Future plans and Discussions

### Awareness of Different Backends

In the future, the system should match the optimal backend automatically based on feedbacks from (both) the configs and the server.

### Markdown Rendering

Optimize the markdown rendering supporting tables.

First choice is **rust native** library for markdown rendering. Sub-optimal user friendly binary-utils: framebuffer or sixel would be potentially choices. But they are neither cross platform nor developer friendly. Considering there is no image rending task currently, we could use the native library for now.

