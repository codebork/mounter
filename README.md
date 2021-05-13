# udman

A utility that manages automounting drives using UDisks2

## Usage

udman runs as a service, its behaviour can be modified using a toml file located
at `$XDG_CONFIG_HOME/udman/config.toml`.

By default udman will notify you of newly attached devices but will not attempt
to automatically mount them. This can be turned on by setting the automount
flag to true in the configuration file.

```toml
   [settings]
   automount=true
```

This setting will apply to all storage devices that are attached. To turn off
automounting for a specific device you can add settings that apply only to
filesystems with a matching UUID.

The UUID for a partition can be found using `lsblk -o +uuid`

```bash
  NAME                MAJ:MIN RM   SIZE RO TYPE  MOUNTPOINT UUID
  sda                   8:0    0  14.5G  0 disk
  └─sda1                8:1    0  14.5G  0 part            a3a0f6ae-aa27-4e0d-9996-8e5cf6756843
```

You can then use the UUID like so

```toml
   [uuid.a3a0f6ae-aa27-4e0d-9996-8e5cf6756843]
   automount=false
```

Other settings that can be set on a filesystem specific level are `run`,
`password` and `keyfile`.

`run` specifies a script file that will be run when the filesystem is
mounted.

`password` and `keyfile` are used to unlock encrypted filesystems. If both are
provided then `keyfile` takes precedence.

If no password or keyfile is specified for an attached encrypted filesystem
then the user will be prompted to enter a password through a dialog box.

## Example config

```toml
   [settings]
   automount=true

   [uuid.a3a0f6ae-aa27-4e0d-9996-8e5cf6756843]
   run='/absolute/path/to/script.sh'
   keyfile='/absolute/path/to/keyfile'
```
