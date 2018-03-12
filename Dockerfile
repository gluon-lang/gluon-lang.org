FROM rust:1.24.0

WORKDIR /usr/src/try_gluon

RUN curl -sL https://deb.nodesource.com/setup_8.x | bash -
RUN apt-get install -y nodejs

RUN apt-get update && apt-get install -y curl apt-transport-https && \
    curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && \
    echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list && \
    apt-get update && apt-get install -y yarn

COPY package.json yarn.lock ./
RUN yarn install
RUN yarn global add webpack-cli
RUN yarn global add elm

COPY . .

RUN webpack-cli
RUN cargo update -p https://github.com/gluon-lang/gluon
RUN cargo install

EXPOSE 8080

CMD ["try_gluon"]
