'use strict';

require('./index.html');
require('./styles.scss');

var Elm = require('./Main.elm');
var mountNode = document.getElementById('main');

var app = Elm.Elm.Main.init({
    node: mountNode,
    flags: {
        origin: document.location.origin,
        pathname: document.location.pathname,
        href: document.location.href,
    },
});
