# Inkdrop plugins

## Install

```shell
cargo install inkdrop-plugins
```

## For alias setup

### Bash

```shell
echo 'alias inkp="inkdrop-plugins"' >> ~/.bashrc

source ~/.bashrc

echo "Installation complete. You can now use the command 'inkp'."
```

### Powershell

```powershell
Add-Content -Path $PROFILE -Value 'Set-Alias inkp inkdrop-plugins'

. $PROFILE

Write-Host "Installation complete. You can now use the command 'inkp'."
```

## Usage

Print inkdrop plugins list.

```shell
inkdrop-plugins --list
```

## LICENSE

MIT

## Author

Daiki Nakashima
