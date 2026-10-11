Name:           zip-lang
Version:        3.0.0
Release:        1%{?dist}
Summary:        ZIP programming language
License:        MIT
URL:            https://github.com/3537mari/zip

%description
ZIP — Cyrillic programming language (compiler + VM).

%install
mkdir -p %{buildroot}/usr/local/bin
# Fill from CI-built binaries

%files
/usr/local/bin/zipc
/usr/local/bin/zip-vm
