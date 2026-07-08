'use strict';

require('./styles.scss');

import('./Main.elm').then(Elm => {
    var mountNode = document.getElementById('main');

    var app = Elm.Elm.Main.init({
        node: mountNode,
        flags: {
            origin: document.location.origin,
            pathname: document.location.pathname,
            href: document.location.href,
        },
    });
});
