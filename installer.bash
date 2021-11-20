#!/bin/bash

LATEST_VERSION="v1.0.0-beta"

OS_TYPE="$YTUI_MUSIC_OS_TYPE"
BIN_SUFFIX="$YTUI_MUSIC_BIN_SUFFIX"
function get_os_type() {
    if [ $OSTYPE=="linux" ] || [ $OS_TYPE=="linux-gnu"]; then
        OS_TYPE="linux"
    elif [ $OSTYPE=="darwin" ]; then
        OS_TYPE="macos"
    elif [ $OSTYPE=="cygwin" ] || [ $OSTYPE=="msys" ] || [ $OSTYPE=="win32" ]; then
        OS_TYPE="windows"
        BIN_SUFFIX=".exe"
    else
        echo "Cannot get operating system type from bash."
        echo "If you OS is not listed here. Refer to https://github.com/sudipghimire533/ytui-music/blob/main/CONTRIBUTING.md#building"
        while [ $OS_TYPE=="" ]; do
            echo "[1] Linux"
            echo "[2] Windows"
            echo "[3] MacOs"
            read -p "Select your operating system (option 1-3) >" RESPONSE
            if [ $RESPONSE=="1" ]; then
                OS_TYPE="linux"
            elif [ $RESPONSE=="2" ]; then
                OS_TYPE="windows"
                BIN_SUFFIX=".exe"
            elif [ $RESPONSE=="3" ]; then
                OS_TYPE="macos"
            else
                echo "Invalid option selected. Try again.."
            fi
        done
    fi
}

function check_config_dir() {
    echo "Locating config directory..."
    local CONFIG_DIR=""

    if [ $HOME!="" ]; then
        CONFIG_DIR="$HOME/.config"
    else
        if [ $XDG_CONFIG_HOME=="" ]; then
            echo "Variable \$XDG_CONFIG_HOME is not set..";
            echo "Either \$HOME or \$XDG_CONFIG_HOME variable is required..."
        else
            CONFIG_DIR=$XDG_CONFIG_HOME
        fi
    fi

    if [ -d $CONFIG_DIR ]; then
        CONFIG_DIR="$CONFIG_DIR/ytui-music"
        echo "Your configuration files will be stored in $CONFIG_DIR"

        if [ -d $CONFIG_DIR ]; then
            echo "Cool! Directory already exists.."
        else
            echo "Creating $CONFIG_DIR ..."
            mkdir $CONFIG_DIR
            if [ $? -eq 0 ]; then
                echo "Ytui-music config directory is ready..."
            else
                echo "Failed to create ytui-music config directory..."
                CONFIG_DIR_LOCATED=false
                return 1
            fi
        fi
    fi

    export YTUI_CONFIG_DIR=$CONFIG_DIR
    return 0
}

# Accepts a paramater in which to save binary to
function download_binary() { # Depends on $SAVE_PATH from get_save_path function
    echo "Using curl to download: $REMOTE_URL"

    type curl
    if [ $? -ne 0 ]; then
        echo "Cannot execute curl command..."
        return 1
    fi

    TEMP_PATH=$(mktemp)
    curl -L --location-trusted $REMOTE_URL > $TEMP_PATH

    if [ $? -ne 0 ]; then
        echo "Cannot download ytui_music binary..."
        return 1
    fi

    mv $TEMP_PATH "$SAVE_PATH"

    if [ $? -ne 0 ]; then
        echo "Cannot move downloadd binary to save location.."
        echo "Try moving manually..."
        return 1
    fi

    return 0
}

SAVE_PATH="$YTUI_MUSIC_SAVE_PATH"
function get_save_path() { # This function depends on the $BIN_SUFFIX variable from get_os_type
    local RESPONSE=""
    while [ $SAVE_PATH=="" ]; do
        read -p "Destination directory on where to save final binary? eg: /usr/local/bin >" RESPONSE
        if [ $RESPONSE != "" ]; then
            SAVE_PATH="$RESPONSE/ytui_music$BIN_SUFFIX"
            break
        else
            echo "Destination cannot be empty. Try again."
        fi
    done
}

CPU_ARCH="$YTUI_MUSIC_CPU_ARCH"
function get_cpu_arch() {
    echo "Scipt need to know which cpu you are running on."
    echo "If your cpu is not listed here. You can refer to https://github.com/sudipghimire533/blob/main/CONTRIBUTING.md#building"
    while [ $CPU_ARCH=="" ]; do
        echo "[1] x86_64 / amd64"
        echo "[2] x86 (32-bit intel)"
        echo "[3] arm"
        echo "[4] aarch64"
        read -p "Select your Cpu type (option 1 to 4) >" RESPONSE
        if [ $RESPONSE=="1" ]; then
            CPU_ARCH="amd64"; break
        elif [ $RESPONSE=="2"]; then
            CPU_ARCH="x86"; break
        elif [ $RESPONSE=="3" ]; then
            CPU_ARCH="arm"; break
        elif [ $RESPONSE=="4" ]; then
            CPU_ARCH="aarch64"; break
        else
            echo "Invalid option selected. Try again."
        fi
    done
}


echo ""
echo "Ytui music installer.."
echo "On sucessfull execution of this script you can use ytui music. Project: https://github.com/sudipghimire33/ytui-music"

echo ""
echo "Checking required directory structure..."
check_config_dir
if [ $? -ne 0 ]; then
    echo "You need to have one of following existing directory:"
    echo "- \$HOME/.config/ytui-music. Currently \$HOME points to $HOME"
    echo "- \$XDG_CONFIG_HOME/ytui-music. Currently \$XDG_CONFIG_HOME points to $XDG_CONFIG_HOME"
    echo "- \$YTUI_CONFIG_DIR. Currently \$YTUI_CONFIG_DIR points to $YTUI_CONFIG_DIR"
fi

echo ""; get_os_type
echo ""; get_save_path # This function assumes that get_os_type is called already
echo ""; get_cpu_arch

REMOTE_URL="https://github.com/sudipghimire533/ytui-music/releases/download/$LATEST_VERSION/ytui_music-$OS_TYPE-$CPU_ARCH""$BIN_SUFFIX"
download_binary # This function assumes that get_save_path function is already called

if [ $OS_TYPE=="linux" ] || [ $OS_TYPE=="macos" ]; then
    chmod +x "$SAVE_PATH"
fi

echo ""
echo "Latest version of ytui-music have been sucessfully installed to $SAVE_PATH"
echo ""
echo "Wishing you happy music - Sudip Ghimire"
echo "Exiting..."
echo ""
