#!/usr/bin/env bash

apt-get update
apt-get -y install --no-install-recommends ruby-full build-essential zlib1g-dev
echo '# Install Ruby Gems to ~/gems' >> ~/.bashrc
echo 'export GEM_HOME="$HOME/gems"' >> ~/.bashrc
echo 'export PATH="$HOME/gems/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
gem install jekyll bundler
apt-get clean -y && rm -rf /var/lib/apt/lists/*