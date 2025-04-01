# Tembo Cloud Apt Packaging

This project builds Apt packages for [Tembo Cloud].

  [Tembo Cloud]: https://cloud.tembo.io

## Repackage Usage

The [`repackage`](bin/repackage) script downloads an Apt package and creates a
tarball with the that contains the shared object libraries to be installed on
the Tembo Cloud persistent volume. Example:

```console
# ./bin/repackage libjson-c libjson-c5
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
