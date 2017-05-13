Simple tool to list disk-usage at path stratified by owner.

## Example output (formatted)

    $ disk_usage /path/to/files/
    User       NFiles    NLinks    Size        Raw             Frac
    user1      1         0         0           0               0.000
    user2      129       0         0           0               0.000
    user3      9         0         12.0 KB     12288           0.000
    user4      14        0         68.0 KB     69632           0.000
    user5      5         0         72.0 KB     73728           0.000
    user6      80        0         381.5 MB    400064512       0.024
    user7      27        0         931.4 MB    976633856       0.059
    user8      2986      0         14.0 GB     15050215424     0.916
    *          3251      0         15.3 GB     16427069440     1.000

## Usage

USAGE:
    disk_usage [FLAGS] [root]...

FLAGS:
        --apparent-size    Calculate apparent size rather than block size.
    -h, --help             Prints help information
    -V, --version          Prints version information

ARGS:
    <root>...    Root folder or file.
