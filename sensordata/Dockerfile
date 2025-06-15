FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y --no-install-recommends git wget unzip curl make xz-utils && \
    apt-get install -y --no-install-recommends python3-pip android-tools-adb android-tools-fastboot && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get autoremove -y && \
    apt-get clean

ARG FRIDA_VERSION=16.2.1
WORKDIR /required-files

RUN wget https://github.com/frida/frida/releases/download/${FRIDA_VERSION}/frida-server-${FRIDA_VERSION}-android-arm64.xz && \
    unxz frida-server-${FRIDA_VERSION}-android-arm64.xz && \
    mv frida-server-${FRIDA_VERSION}-android-arm64 frida-server

RUN curl -fsSL https://bun.sh/install | bash

ENV PATH ${PATH}:~/.bun/bin

RUN pip install --break-system-packages frida-tools==13

ADD https://assets.anurag.sh/com.mcdonalds.au.gma.apk app.apk
COPY getsensordata.js .
COPY run.sh .
COPY server.js .

ENTRYPOINT ["/bin/bash", "run.sh"]
EXPOSE 8080
