const path = require("path");
const ClosureCompilerPlugin = require('webpack-closure-compiler');

module.exports = {
  entry: {
    'try/app': [
      './src/client/try/index.js'
    ],
    'gluon-lang': [
        './src/client/index.js'
    ]
  },

  output: {
    path: path.resolve(__dirname + '/dist'),
    filename: '[name].js',
  },

  module: {
    rules: [
      {
        test: /\.(css|scss)$/,
        use: [
          'style-loader',
          'css-loader',
          'sass-loader',
        ]
      },
      {
        test: /\.html$/,
        exclude: /node_modules/,
        use: {
          loader: 'file-loader',
          options: {
            name: (file) => {
                let prefix = /src[\/\\]client(.*[\/\\]).+$/.exec(file)[1]
                return prefix + '[name].[ext]';
            }
          }
        }
      },
      {
        test: /\.elm$/,
        exclude: [/elm-stuff/, /node_modules/],
        use: 'elm-webpack-loader',
      },
      {
        test: /\.woff(2)?(\?v=[0-9]\.[0-9]\.[0-9])?$/,
        use: 'url-loader?limit=10000&mimetype=application/font-woff',
      },
      {
        test: /\.(ttf|eot|svg)(\?v=[0-9]\.[0-9]\.[0-9])?$/,
        use: 'file-loader',
      },
    ],

    noParse: /\.elm$/,
  },

  devServer: {
    inline: true,
    stats: { colors: true },
  },

  plugins: [
    new ClosureCompilerPlugin({
      jsCompiler: true,
      compiler: {
        compilation_level: 'SIMPLE',
        create_source_map: false,
        language_out: 'ECMASCRIPT5',
      },
    }),
  ],
};
