FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y --no-install-recommends openjdk-17-jdk && \
    apt-get install -y --no-install-recommends git wget unzip curl make xz-utils && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get autoremove -y && \
    apt-get clean

ENV ANDROID_HOME=/android-sdk
ENV ANDROID_SDK_HOME $ANDROID_HOME
ENV ANDROID_SDK_ROOT $ANDROID_HOME

ARG ANDROID_SDK_VERSION=11479570
ARG ANDROID_BUILD_TOOLS_VERSION=34.0.0

RUN mkdir -p ${ANDROID_SDK_HOME}/cmdline-tools && \
    wget -q https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_VERSION}_latest.zip && \
    unzip *tools*linux*.zip -d ${ANDROID_SDK_HOME}/cmdline-tools && \
    rm *tools*linux*.zip

RUN mv ${ANDROID_SDK_HOME}/cmdline-tools/cmdline-tools ${ANDROID_SDK_HOME}/cmdline-tools/latest

ENV PATH ${PATH}:${ANDROID_SDK_HOME}/tools:${ANDROID_SDK_HOME}/platform-tools

ARG FRIDA_VERSION=16.2.1
WORKDIR /required-files

RUN wget https://github.com/frida/frida/releases/download/${FRIDA_VERSION}/frida-server-${FRIDA_VERSION}-android-arm64.xz && \
    unxz frida-server-${FRIDA_VERSION}-android-arm64.xz && \
    mv frida-server-${FRIDA_VERSION}-android-arm64 frida-server

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y --no-install-recommends python3-pip android-tools-adb android-tools-fastboot && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get autoremove -y && \
    apt-get clean

RUN curl -fsSL https://bun.sh/install | bash

ENV PATH ${PATH}:~/.bun/bin

RUN pip install --break-system-packages frida-tools==13

ADD https://assets.anurag.sh/com.mcdonalds.au.gma.apk app.apk
COPY getsensordata.js .
COPY run.sh .
COPY server.js .

ENTRYPOINT ["/bin/bash", "run.sh"]
EXPOSE 8080
