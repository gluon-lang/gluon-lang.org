# gluon-lang.org

[![Build Status](https://travis-ci.org/gluon-lang/gluon-lang.org.svg?branch=master)](https://travis-ci.org/gluon-lang/gluon-lang.org) [![Gitter](https://badges.gitter.im/gluon-lang/gluon.svg)](https://gitter.im/gluon-lang/gluon?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

Source for the https://gluon-lang.org website which contains the documentation for the [Gluon](https://github.com/gluon-lang/gluon)
programming language as well as a service which lets you run gluon scripts from your browser.

To run the server:

```
npm install
npm install -g webpack
npm install -g elm
webpack
cargo run
```

You can also run the webpack watcher using:

```
webpack --watch
```
