#!/usr/bin/bash

if [ -n $TERMUX_VERSION ]; then
    apt update
    yes | pkg install -y git rust termux-elf-cleaner p7zip 2>/dev/null | grep -E '(Need to get |Get:|Unpacking |Setting up )'
else
    echo "The script should run on Termux"
    exit 1
fi

rustc --version
cargo --version

current_dir="$(pwd)"
tmpdir="$(mktemp -d)"

cd $tmpdir

git clone https://github.com/IMXEren/competition-groups

if [ -n $TERM ]; then
    clear
fi

# fix Termux permissions
if [ -n $TERMUX_VERSION ]; then
    value="true"; key="allow-external-apps"; file="/data/data/com.termux/files/home/.termux/termux.properties"; mkdir -p "$(dirname "$file")"; chmod 700 "$(dirname "$file")"; if ! grep -E '^'"$key"'=.*' $file &>/dev/null; then [[ -s "$file" && ! -z "$(tail -c 1 "$file")" ]] && newline=$'\n' || newline=""; echo "$newline$key=$value" >> "$file"; else sed -i'' -E 's/^'"$key"'=.*/'"$key=$value"'/' $file; fi
fi

echo -e "\nFinal step, building compgroups binary. Takes about 10s~1min"

package_script='#!/system/bin/sh

dir="$(cd "$(dirname "$0")"; pwd)"
bin_name="$(basename "$0")"
chmod 744 "$0" &>/dev/null
chmod 744 "$dir/${bin_name}.bin" &>/dev/null
export LD_LIBRARY_PATH="$dir/lib-${bin_name}:$LD_LIBRARY_PATH"

if [ $(getprop ro.build.version.sdk) -gt 28 ]; then
	if getprop ro.product.cpu.abilist | grep -q "64"
	then
    	exec /system/bin/linker64 "$dir/${bin_name}.bin" "$@"
	else
    	exec /system/bin/linker "$dir/${bin_name}.bin" "$@"
	fi
else
	exec "$dir/${bin_name}.bin" "$@"
fi'

yes | pkg install -y pkg-config openssl binutils
cd competition-groups
cargo build --release
bin_path="${tmpdir}/competition-groups/target/release/competition-groups"
package="compgroups"
zip_name="cgroups.zip"

if [ $? -eq 0 ]; then
    if [ -n $TERMUX_VERSION ]; then
        termux-elf-cleaner ${tmpdir}/cgroups/${package}.bin &>/dev/null
    fi
    cd $current_dir
    rm -rf build/${zip_name} build/${package} build/${package}.bin build/lib-${package} &>/dev/null
    mkdir -p build
    cd build
    cp ${tmpdir}/cgroups/${package}.bin .
    if [ $? -ne 0 ]; then
        rm -rf $tmpdir &>/dev/null
        echo "Error occured, exiting..."
	exit 1
    fi
    echo "$package_script" > ${package}
    for libpath in $(ldd ${package}.bin | grep -F "/data/data/com.termux/" | sed "s/.* //g"); do
        cp -L "$libpath" lib-${package} &>/dev/null
    done
    echo -e "\n  Zipping package..."
    chmod 744 ${package} ${package}.bin &>/dev/null
    7z a -tzip -mx=9 -bd -bso0 ${zip_name} ${package} ${package}.bin lib-${package}
    rm -rf ${package} ${package}.bin lib-${package} &>/dev/null
else
    rm -rf $tmpdir &>/dev/null
    echo "Error occured, exiting..."
    exit 1
fi

rm -rf $tmpdir &>/dev/null


if [ -n $TERMUX_VERSION ]; then
    pkg clean
fi

mkdir -p ~/cgroups

7z x -aoa ${zip_name} -o$HOME/cgroups &>/dev/null

chmod 744 ~/cgroups/compgroups

echo -e "All done! You can type this -\n  \n\" cd ~/cgroups; ./compgroups \"\n\nType without quotes to run executable\n"

