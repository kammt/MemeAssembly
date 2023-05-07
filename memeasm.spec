Name:           memeasm
Version:        0.0.0
Release:        0%{?dist}
Summary:        A Meme-based programming language

License:        GPLv3
URL:            https://github.com/kammt/MemeAssembly
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  make
Requires:       gcc

%description
The compiler for MemeAssembly, a Meme-based programming language.
Learn more at https://kammt.github.io/MemeAssembly.

%prep
%autosetup


%build
%make_build


%install
%make_install


%files
%license LICENSE



%changelog
