const path = require("path");
const ClosureCompilerPlugin = require('closure-compiler-webpack-plugin');

module.exports = {
  entry: {
    app: [
      './src/client/index.js'
    ]
  },

  output: {
    path: path.resolve(__dirname + '/dist/try'),
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
        use: 'file-loader?name=[name].[ext]',
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
      compilation_level: 'SIMPLE',
      create_source_map: false,
      language_out: 'ECMASCRIPT5',
    }),
  ],
};
