# Tembo Cloud Apt Packaging

This project builds Apt packages for [Tembo Cloud].

  [Tembo Cloud]: https://cloud.tembo.io

## Repackage Usage

The [`repackage`](bin/repackage) script downloads an Apt package and creates a
new package with the prefix `tembo-` that simply installs shared object
libraries into the Tembo Cloud persistent volume. Example:

```console
# ./bin/repackage libjson-c5
Working in /tmp/tmp.SLZnJ03uyK
Get:1 http://ports.ubuntu.com/ubuntu-ports noble/main arm64 libjson-c5 arm64 0.17-1build1 [36.4 kB]
Fetched 36.4 kB in 1s (71.2 kB/s)  
./bin/repackage: line 69: DEBIAN/md5sums: No such file or directory
root@d99fa2f3a489:/work# ./bin/repackage libjson-c5
Working in /tmp/tmp.kWmF8szfQA
Get:1 http://ports.ubuntu.com/ubuntu-ports noble/main arm64 libjson-c5 arm64 0.17-1build1 [36.4 kB]
Fetched 36.4 kB in 1s (61.5 kB/s)     
dpkg-deb: building package 'tembo-libjson-c5' in 'tembo-libjson-c5_0.17-1build1_arm64.deb'.
 new Debian package, version 2.0.
 size 30456 bytes: control archive=429 bytes.
     373 bytes,     9 lines      control
      81 bytes,     1 lines      md5sums
 Package: tembo-libjson-c5
 Architecture: arm64
 Version: 0.17-1build1
 Depends: libc6 (>= 2.38)
 Maintainer: Tembo <admin+apt@tembo.io>
 Description: JSON manipulation library - shared library
  This library allows you to easily construct JSON objects in C,
  output them as JSON formatted strings and parse JSON formatted
  strings back into the C representation of JSON objects.
drwxr-xr-x root/root         0 2025-03-28 22:09 ./
drwxr-xr-x root/root         0 2025-03-28 22:09 ./var/
drwxr-xr-x root/root         0 2025-03-28 22:09 ./var/lib/
drwxr-xr-x root/root         0 2025-03-28 22:09 ./var/lib/postgresql/
drwxr-xr-x root/root         0 2025-03-28 22:09 ./var/lib/postgresql/data/
drwxr-xr-x root/root         0 2025-03-28 22:09 ./var/lib/postgresql/data/lib/
-rw-r--r-- root/root    133288 2025-03-28 22:09 ./var/lib/postgresql/data/lib/libjson-c.so.5.3.0
lrwxrwxrwx root/root         0 2025-03-28 22:09 ./var/lib/postgresql/data/lib/libjson-c.so.5 -> libjson-c.so.5.3.0
```
