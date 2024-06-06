FROM emscripten/emsdk:3.1.55

RUN apt-get update -qq -y
RUN apt-get -qq install -y --no-install-recommends \
  wabt pkg-config libssl-dev libreadline-dev nano \
  && apt-get -y clean \
  && apt-get -y autoclean \
  && apt-get -y autoremove

ENV LUA_VERSION 5.3.4
ENV LUAROCKS_VERSION 3.11.0
ENV NODE_VERSION=18.17.0

RUN pip3 install pyyaml

# Install lua runtime
RUN cd / \
  && curl -L http://www.lua.org/ftp/lua-${LUA_VERSION}.tar.gz | tar xzf - \
  && cd /lua-${LUA_VERSION} \
  && make linux test \
  && make install

# Install luarocks
RUN cd / && \
  curl -L https://luarocks.org/releases/luarocks-${LUAROCKS_VERSION}.tar.gz | tar xzf - && \
  cd /luarocks-${LUAROCKS_VERSION} && \
  ./configure && \
  make build && \
  make install

# Install NodeJS
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"

RUN cd /root && curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain add stable \
  && rustup target add wasm32-unknown-emscripten --toolchain stable \
  && rustup target add nightly-x86_64-unknown-linux-gnu --toolchain
RUN rustup toolchain add nightly \
  && rustup target add wasm32-unknown-emscripten --toolchain nightly \
  && rustup target add nightly-x86_64-unknown-linux-gnu --toolchain nightly

# # Install Binaryen
# RUN cd /root \
#   && git clone https://github.com/WebAssembly/binaryen.git \
#   && cd binaryen \
#   && git submodule init \
#   && git submodule update \
#   && cmake . && make
# ENV PATH="/root/binaryen/bin:${PATH}"
