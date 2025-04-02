# Tembo Cloud Packaging

This project builds dependency packages for [Tembo Cloud].

## Usage

1.  Make sure the package database is up-to-date:

    ``` sh
    apt-get update
    ```

2.  Use `make -j` to build all the packages in parallel across all CPUs.

    ``` sh
    make -j$(nproc)
    ```

3.  Use `make install` to install the packages:

    ``` sh
    make -j$(nproc) install
    ```

## `repackage` Usage

The [`repackage`](bin/repackage) script downloads an Apt package and creates a
tarball that contains the shared object libraries to be installed on the Tembo
Cloud persistent volume. The package must have a config file,
`packages/$package.cfg`. Example:

```console
# ./bin/repackage libjson-c
Working in /tmp/tmp.HLNvAFmsMY
Get:1 http://ports.ubuntu.com/ubuntu-ports noble/main arm64 libjson-c5 arm64 0.17-1build1 [36.4 kB]
Fetched 36.4 kB in 1s (57.6 kB/s)     
'libjson-c5/usr/lib/aarch64-linux-gnu/libjson-c.so.5.3.0' -> 'tembo-libjson-c_arm64/lib/libjson-c.so.5.3.0'
'libjson-c5/usr/lib/aarch64-linux-gnu/libjson-c.so.5' -> 'tembo-libjson-c_arm64/lib/libjson-c.so.5'
'libjson-c5/usr/share/doc/libjson-c5/copyright' -> 'tembo-libjson-c_arm64/doc/copyright'
tembo-libjson-c_arm64/
tembo-libjson-c_arm64/digests
tembo-libjson-c_arm64/lib/
tembo-libjson-c_arm64/lib/libjson-c.so.5.3.0
tembo-libjson-c_arm64/lib/libjson-c.so.5
tembo-libjson-c_arm64/doc/
tembo-libjson-c_arm64/doc/copyright
```

## `install_package` Usage

Once the tarball has been built for a package, use
[`install_package`](bin/install_package) to install it:

``` console
# ./bin/install_package libjson-c
tembo-libjson-c_arm64/
tembo-libjson-c_arm64/digests
tembo-libjson-c_arm64/lib/
tembo-libjson-c_arm64/lib/libjson-c.so.5.3.0
tembo-libjson-c_arm64/lib/libjson-c.so.5
tembo-libjson-c_arm64/doc/
tembo-libjson-c_arm64/doc/copyright
./lib/libjson-c.so.5.3.0: OK
./doc/copyright: OK
removed '/var/lib/postgresql/data/lib/libjson-c.so.5'
'lib/libjson-c.so.5' -> '/var/lib/postgresql/data/lib/libjson-c.so.5'
'lib/libjson-c.so.5.3.0' -> '/var/lib/postgresql/data/lib/libjson-c.so.5.3.0'
```

  [Tembo Cloud]: https://cloud.tembo.io
