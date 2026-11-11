Name:           memeasm
Version:        1.6
Release:        1%{?dist}
Summary:        A Meme-based programming language

License:        GPLv3
URL:            https://github.com/kammt/MemeAssembly
Source0:        %{name}-%{version}.tar.gz

# keep LTO flags away from the C code compiled by build.rs
%global _lto_cflags %{nil}

BuildRequires:  cargo
BuildRequires:  rust
# for the C code compiled by build.rs
BuildRequires:  gcc
BuildRequires:  make
# memeasm writes executables itself; gcc is only needed to link object files created with -O
Suggests:       gcc

%description
The compiler for MemeAssembly, a Meme-based programming language.
Learn more at https://kammt.github.io/MemeAssembly.

%prep
%autosetup


%build
%global debug_package %{nil}
%make_build CARGO_FLAGS=--locked


%install
%make_install CARGO_FLAGS=--locked bindir=%{_bindir}


%files
%license LICENSE
%{_bindir}/memeasm


%changelog
* Sun May 07 2023 tobias <tobias.kamm@pm.me> 1.6-1
- new package built with tito

