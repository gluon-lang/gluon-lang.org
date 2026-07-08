const path = require("path");
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

module.exports = function (env, args) {
  return {
    entry: {
      'try/app': [
        './src/client/try/index.js'
      ],
      'gluon-lang': [
          './src/client/index.js'
      ]
    },

    mode: args.mode,

    output: {
      path: path.resolve(__dirname + '/target/dist'),
      filename: '[name].js',
    },

    module: {
      rules: [
        {
          test: /\.(css|scss)$/,
          exclude: [/elm-stuff/, /node_modules/],
          use: [
            MiniCssExtractPlugin.loader,
            'css-loader',
            {
              loader: 'sass-loader',
              options: {
                sassOptions: {
                  quietDeps: true,
                },
              },
            },
          ]
        },
        {
          test: /\.elm$/,
          exclude: [/elm-stuff/, /node_modules/],
          use: {
              loader: 'elm-webpack-loader',
              options: {
                  optimize: args.mode == 'production',
              }
          },
        },
        {
          test: /\.woff(2)?(\?v=[0-9]\.[0-9]\.[0-9])?$/,
          exclude: [/elm-stuff/, /node_modules/],
          type: 'asset',
          parser: {
            dataUrlCondition: {
              maxSize: 10000,
            },
          },
        },
        {
          test: /\.(ttf|eot|svg)(\?v=[0-9]\.[0-9]\.[0-9])?$/,
          exclude: [/elm-stuff/, /node_modules/],
          type: 'asset/resource',
        },
      ],

      noParse: /\.elm$/,
    },

    plugins: [
      new HtmlWebpackPlugin({
        filename: 'index.html',
        template: 'src/client/index.html',
        chunks: ['gluon-lang'],
      }),
      new HtmlWebpackPlugin({
        filename: 'try/index.html',
        template: 'src/client/try/index.html',
        chunks: ['try/app'],
      }),
      new HtmlWebpackPlugin({
        filename: '404.html',
        template: 'src/client/404.html',
        inject: false,
      }),
      new MiniCssExtractPlugin({
        filename: '[name].css',
      }),
    ],

    watchOptions: {
        ignored: /node_modules/
    }
  }
};
