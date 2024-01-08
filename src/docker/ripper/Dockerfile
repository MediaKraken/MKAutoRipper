# Download base image
FROM mediakraken/mkbase_rust_debian:dev as planner-mkripper

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

####################################################################################################
## Build from Chef
####################################################################################################
FROM planner-mkripper AS cargo-build-mkripper

COPY --from=planner-mkripper recipe.json recipe.json

RUN cargo chef cook --release --target x86_64-unknown-linux-gnu --recipe-path recipe.json

WORKDIR /mediakraken

COPY ./ .

RUN cargo build --release --target x86_64-unknown-linux-gnu \
  && cp /mediakraken/target/x86_64-unknown-linux-gnu/release/mk* /mediakraken/target/.;

RUN ldd /mediakraken/target/mkripper | tr -s '[:blank:]' '\n' | grep '^/' | \
  xargs -I % sh -c 'mkdir -p $(dirname /mediakraken/deps%); cp % /mediakraken/deps%;'

####################################################################################################
## Final image
####################################################################################################
FROM scratch
# otherwise throws file not found error
ADD rootfs.tar.xz /
# deps has full /usr/lib/x86_64-linux-gnu path.....hence copy to /
COPY --from=cargo-build-mkripper /mediakraken/deps/* /

# Import from builder.
COPY --from=cargo-build-mkripper /wait /wait

# Copy our build
COPY --from=cargo-build-mkripper /mediakraken/target/mkripper /mkripper

WORKDIR /mediakraken
ADD https://www.makemkv.com/download/makemkv-bin-1.17.5.tar.gz /mediakraken/.
ADD https://www.makemkv.com/download/makemkv-oss-1.17.5.tar.gz /mediakraken/.

# qtbase5-dev
RUN tar -xvzf /mediakraken/makemkv-bin* \
  && tar -xvzf /mediakraken/makemkv-oss* \
  && apt-get update && apt-get install -y \
  build-essential pkg-config libc6-dev libssl-dev libexpat1-dev libavcodec-dev libgl1-mesa-dev zlib1g-dev \
  abcde \
  libcdparanoia0 \
  less coreutils \
  && rm -rf /var/lib/apt/lists/* \
  && cd /mediakraken/makemkv-oss-1.17.5 && yes accept | ./configure --disable-gui && make -j`getconf _NPROCESSORS_ONLN` && make install \
  && sed -i '2i exit 0;' /mediakraken/makemkv-bin-1.17.5/src/ask_eula.sh \
  && cd /mediakraken/makemkv-bin-1.17.5 && make -j`getconf _NPROCESSORS_ONLN` && make install \
  && rm -Rf /mediakraken/makemkv-*
 