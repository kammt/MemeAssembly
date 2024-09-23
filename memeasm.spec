Name:           memeasm
Version:        1.6
Release:        1%{?dist}
Summary:        A Meme-based programming language

License:        GPLv3
URL:            https://github.com/kammt/MemeAssembly
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  cmake
BuildRequires:  ninja-build

%description
The compiler for MemeAssembly, a Meme-based programming language.
Learn more at https://kammt.github.io/MemeAssembly.

%prep
%autosetup  

%build
%global debug_package %{nil}
%cmake
%cmake_build


%install
%cmake_install


%files
%license LICENSE
/usr/local/bin/memeasm


%changelog
* Sun May 07 2023 tobias <tobias.kamm@pm.me> 1.6-1
- new package built with tito

