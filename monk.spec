Name:           monk
Version:        0.2m0
Release:        1
Summary:        A cli for storing webpages for future consumption.
License:        AGPL-3.0
URL:            https://gitlab.com/fisherdarling/monk

Requires:       openssl

#BuildRequires:  cargo
#BuildRequires:  gcc
#BuildRequires:  openssl-devel


%description
A cli for storing and searching webpages.

%build
# Binaries will already be build by the CICD Pipeline, no need to do it again. 
#cargo build --release

%install
mkdir -p %{buildroot}/usr/bin/
install -m 755 target/release/monk %{buildroot}/usr/bin/monk
install -m 755 target/release/monkd %{buildroot}/usr/bin/monkd

%files
/usr/bin/monk
/usr/bin/monkd
