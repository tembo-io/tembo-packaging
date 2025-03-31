# Tembo Cloud Apt Packaging

This project builds Apt packages for [Tembo Cloud].

  [Tembo Cloud]: https://cloud.tembo.io

## Repackage Usage

The [`repackage`](bin/repackage) script downloads an Apt package and creates a
new package with the prefix `tembo-` that simply installs shared object
libraries into the Tembo Cloud persistent volume. Example:

```console
# ./bin/repackage libjson-c  libjson-c5
Working in /tmp/tmp.P9d3vIvUBL
Get:1 http://ports.ubuntu.com/ubuntu-ports noble/main arm64 libjson-c5 arm64 0.17-1build1 [36.4 kB]
Fetched 36.4 kB in 0s (102 kB/s)
dpkg-deb: building package 'tembo-libjson-c' in 'tembo-libjson-c_0.17-1build1_arm64.deb'.
 new Debian package, version 2.0.
 size 30450 bytes: control archive=432 bytes.
     372 bytes,     9 lines      control
      81 bytes,     1 lines      md5sums
 Package: tembo-libjson-c
 Architecture: arm64
 Version: 0.17-1build1
 Depends: libc6 (>= 2.38)
 Maintainer: Tembo <admin+apt@tembo.io>
 Description: JSON manipulation library - shared library
  This library allows you to easily construct JSON objects in C,
  output them as JSON formatted strings and parse JSON formatted
  strings back into the C representation of JSON objects.
drwxr-xr-x root/root         0 2025-03-31 18:42 ./
drwxr-xr-x root/root         0 2025-03-31 18:42 ./var/
drwxr-xr-x root/root         0 2025-03-31 18:42 ./var/lib/
drwxr-xr-x root/root         0 2025-03-31 18:42 ./var/lib/postgresql/
drwxr-xr-x root/root         0 2025-03-31 18:42 ./var/lib/postgresql/data/
drwxr-xr-x root/root         0 2025-03-31 18:42 ./var/lib/postgresql/data/lib/
-rw-r--r-- root/root    133288 2025-03-31 18:42 ./var/lib/postgresql/data/lib/libjson-c.so.5.3.0
lrwxrwxrwx root/root         0 2025-03-31 18:42 ./var/lib/postgresql/data/lib/libjson-c.so.5 -> libjson-c.so.5.3.0
```
