module Main exposing (main)

import Html exposing (Html, a, button, div, form, h2, li, nav, option, pre, select, text, textarea, ul)
import Html.Attributes exposing (class, disabled, href, name, rows, selected, id)
import Html.Events exposing (onClick, onInput)
import Navigation exposing (Location)
import Http
import Json.Decode as Json
import Json.Encode as JsonEncode
import List exposing ((::))
import List.Extra as List
import Dict
import UrlParser exposing (s, parsePath, stringParam, (<?>), top)


-- MODEL


type Response value
    = Pending
    | Fail String
    | Succeed value
    | GistReceived PostGist


type alias Urls =
    { config : String
    , eval : String
    , format : String
    , currentOrigin : String
    }


type alias Example =
    { name : String
    , src : String
    }

type alias Config =
    { lastRelease : String
    , gitMaster : String
    , examples : List Example
    }

type Version = LastRelease | GitMaster

lastReleaseString : Config -> String
lastReleaseString config = "Release: " ++ config.lastRelease

versionsMap : Config -> List (Version, String)
versionsMap config = [
        (LastRelease, lastReleaseString config),
        (GitMaster, "Revision: " ++ config.gitMaster)
    ]

humanReadableVersion : Config -> Version -> String
humanReadableVersion config version =
    List.filterMap (\(v, s) -> if version == v then Just s else Nothing) (versionsMap config)
        |> List.head
        |> Maybe.withDefault (lastReleaseString config)

versionFromString : Config -> String -> Version
versionFromString config stringVersion =
    List.filterMap (\(v, s) -> if stringVersion == s then Just v else Nothing) (versionsMap config)
        |> List.head
        |> Maybe.withDefault LastRelease

type alias Model =
    { urls : Urls
    , config : Config
    , selectedExample : Maybe String
    , selectedVersion : Version
    , src : String
    , evalResult : Response String
    }


init : Location -> ( Model, Cmd Msg )
init location =
    let
        model =
            { urls =
                { config = "config"
                , eval = "eval"
                , format = "format"
                , currentOrigin = location.origin ++ location.pathname
                }
            , config = { gitMaster = "Git master", lastRelease = "Last crates.io release", examples = [] }
            , selectedExample = Nothing
            , selectedVersion = LastRelease
            , src = ""
            , evalResult = Succeed ""
            }
    in
        ( model
        , case parsePath (s "try" <?> stringParam "gist") location of
            Just (Just gistId) ->
                Cmd.batch [ getConfig model, loadGist gistId ]

            _ ->
                getConfig model
        )


initConfig : Config -> Model -> Model
initConfig config model =
    let
        newModel = { model | config = config }
    in case List.head config.examples of
        Just example ->
            setExample example.name newModel

        Nothing ->
            newModel


getExample : String -> Model -> Maybe String
getExample name model =
    model.config.examples
        |> List.find (\example -> example.name == name)
        |> Maybe.map .src


setExample : String -> Model -> Model
setExample name model =
    let
        ( selectedExample, src ) =
            case getExample name model of
                Just src ->
                    ( Just name, src )

                Nothing ->
                    ( Nothing, model.src )
    in
        { model | selectedExample = selectedExample, src = src }


setSource : String -> Model -> Model
setSource src model =
    { model | src = src, selectedExample = Nothing }



-- MESSAGES


type Msg
    = EvalRequested
    | EvalDone (Result Http.Error String)
    | ConfigDone (Result Http.Error Config)
    | SelectExample String
    | SelectVersion Version
    | EditSource String
    | FormatRequested
    | FormatDone (Result Http.Error String)
    | GistGetDone (Result Http.Error Gist)
    | Share
    | GistPostDone (Result Http.Error PostGist)
    | NoOp



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        EvalRequested ->
            ( { model | evalResult = Pending }, postEval model )

        EvalDone (Ok result) ->
            ( { model | evalResult = Succeed result }, Cmd.none )

        EvalDone (Err err) ->
            ( { model | evalResult = Fail "Http Error." }, Cmd.none )

        ConfigDone (Ok config) ->
            ( initConfig config model, Cmd.none )

        ConfigDone (Err _) ->
            ( model, Cmd.none )

        SelectExample name ->
            ( setExample name model, Cmd.none )

        SelectVersion version ->
            ( { model | selectedVersion = version }, Cmd.none )

        EditSource src ->
            ( setSource src model, Cmd.none )

        FormatRequested ->
            ( { model | evalResult = Pending }, postFormat model )

        FormatDone (Ok result) ->
            ( { model | src = result, evalResult = Succeed "Formatted!" }, Cmd.none )

        FormatDone (Err err) ->
            ( { model | evalResult = Fail "Unable to format source" }, Cmd.none )

        GistGetDone (Ok gist) ->
            ( { model | src = gist.code }, Cmd.none )

        GistGetDone (Err err) ->
            ( { model | evalResult = Fail ("Unable to load gist: " ++ toString err) }, Cmd.none )

        Share ->
            ( { model | evalResult = Pending }, postGist model )

        GistPostDone (Ok gist) ->
            ( { model | evalResult = GistReceived gist }, Cmd.none )

        GistPostDone (Err err) ->
            ( { model | evalResult = Fail ("Unable to make gist: " ++ toString err) }, Cmd.none )

        NoOp ->
            ( model, Cmd.none )



-- VIEW


exampleSelect : Model -> Html Msg
exampleSelect model =
    let
        selectedAttr key =
            selected (model.selectedExample == key)

        exampleOption example =
            option [ name example.name, selectedAttr (Just example.name) ]
                [ text example.name ]

        defaultOption =
            option [ selectedAttr Nothing ] [ text "Select example…" ]
    in
        div [ class "form pull-xs-right" ]
            [ select [ class "form-control custom-select", onInput SelectExample ]
                (defaultOption :: List.map exampleOption model.config.examples)
            ]


editor : Model -> Html Msg
editor model =
    textarea [ class "editor form-control", rows 25, onInput EditSource ]
        [ text model.src ]


versionSelect : Model -> Html Msg
versionSelect model =
    let
        selectedAttr key =
            selected (model.selectedVersion == key)

        exampleOption version =
            option [ id (toString version), selectedAttr version ]
                [ text (humanReadableVersion model.config version) ]

        defaultOption =
            option [ id (toString LastRelease), selectedAttr LastRelease ] [ text (humanReadableVersion model.config LastRelease) ]
    in
        div [ class "form float-xs-right" ]
            [ select [ class "form-control custom-select", onInput (SelectVersion << versionFromString model.config) ]
                (List.map exampleOption [LastRelease, GitMaster])
            ]



evalResult : Model -> Html Msg
evalResult model =
    let
        evalResult =
            case model.evalResult of
                Pending ->
                    pre [] [ text "Waiting..." ]

                Fail err ->
                    pre [] [ text err ]

                Succeed output ->
                    pre [] [ text output ]

                GistReceived gist ->
                    div []
                        [ a [ href (model.urls.currentOrigin ++ "/?gist=" ++ gist.id) ] [ text "Link to try_gluon" ]
                        , Html.br [] []
                        , a [ href gist.url ] [ text "Link to gist" ]
                        ]
    in
        div [ class "card" ]
            [ div [ class "card-header" ]
                [ nav [ class "nav" ]
                    [ ul [ class "nav navbar-nav mr-auto" ]
                        [ text "Result"
                        ]
                    , ul [ class "nav navbar-nav" ]
                        [ li [ class "nav-item" ]
                            [ versionSelect model
                            ]
                        ]
                    , ul [ class "nav navbar-nav" ]
                        [ li [ class "nav-item" ]
                            [ button
                                [ class "btn btn-secondary float-xs-right"
                                , onClick Share
                                , disabled (model.evalResult == Pending)
                                ]
                                [ text "Share" ]
                            ]
                        ]
                    , ul [ class "nav navbar-nav" ]
                        [ li [ class "nav-item" ]
                            [ button
                                [ class "btn btn-secondary float-xs-right"
                                , onClick FormatRequested
                                , disabled (model.evalResult == Pending)
                                ]
                                [ text "Format" ]
                            ]
                        ]
                    , ul [ class "nav navbar-nav" ]
                        [ li [ class "nav-item" ]
                            [ button
                                [ class "btn btn-primary float-xs-right"
                                , onClick EvalRequested
                                , disabled (model.evalResult == Pending)
                                ]
                                [ text "Eval" ]
                            ]
                        ]
                    ]
                ]
            , div [ class "card-block" ] [ evalResult ]
            ]


view : Model -> Html Msg
view model =
    div [ class "container" ]
        [ Html.br [] []
        , div [ class "card" ]
            [ div [ class "card-header" ]
                [ nav [ class "navbar" ]
                    [ div [ class "navbar-brand" ] [ text "Try Gluon" ]
                    , exampleSelect model
                    ]
                ]
            , div [ class "card-block" ]
                [ editor model
                , Html.br [] []
                , evalResult model
                ]
            , div [ class "card-footer text-muted text-xs-center" ]
                [ a [ href "https://github.com/gluon-lang/gluon" ] [ text "Gluon on Github" ]
                , text " | "
                , a [ href "https://github.com/gluon-lang/try_gluon" ] [ text "Fork this site" ]
                ]
            ]
        ]



-- HTTP


prefixVersion : Model -> String -> String
prefixVersion model path =
    case model.selectedVersion of
        GitMaster -> "master/" ++ path
        LastRelease -> path

postEval : Model -> Cmd Msg
postEval model =
    Http.send EvalDone <|
        Http.post (prefixVersion model model.urls.eval) (Http.stringBody "text/plain" model.src) Json.string


postFormat : Model -> Cmd Msg
postFormat model =
    Http.send FormatDone <|
        Http.post (prefixVersion model model.urls.format) (Http.stringBody "text/plain" model.src) Json.string


getConfig : Model -> Cmd Msg
getConfig model =
    let
        exampleOption =
            Json.map2 (\name value -> { name = name, src = value })
                (Json.field "name" Json.string)
                (Json.field "value" Json.string)

        decodeExamples =
            Json.list exampleOption
        
        decodeConfig =
            Json.map3 (\git last examples -> { gitMaster = git, lastRelease = last, examples = examples })
                (Json.field "git_master" Json.string)
                (Json.field "last_release" Json.string)
                (Json.field "examples" decodeExamples)
    in
        Http.send ConfigDone <|
            Http.get model.urls.config decodeConfig


type alias Gist =
    { id : String, url : String, code : String }


type alias PostGist =
    { id : String, url : String }


baseGistString : String
baseGistString =
    "https://api.github.com/gists"


loadGist : String -> Cmd Msg
loadGist tag =
    let
        files =
            Json.andThen
                (\dict ->
                    case List.head <| Dict.values dict of
                        Nothing ->
                            Json.fail "No files found in gist"

                        Just content ->
                            Json.succeed content
                )
                (Json.dict
                    (Json.field "content" Json.string)
                )

        gistOption =
            Json.map3 (\id url code -> { id = id, url = url, code = code })
                (Json.field "id" Json.string)
                (Json.field "html_url" Json.string)
                (Json.field "files" files)
    in
        Http.send GistGetDone <|
            Http.get (baseGistString ++ "/" ++ tag) gistOption


postGist : Model -> Cmd Msg
postGist model =
    let
        body =
            JsonEncode.object
                [ ( "description", JsonEncode.string "Gluon code shared from try_gluon" )
                , ( "public", JsonEncode.bool True )
                , ( "files"
                  , JsonEncode.object
                        [ ( "try_gluon.glu"
                          , JsonEncode.object
                                [ ( "content", JsonEncode.string model.src )
                                ]
                          )
                        ]
                  )
                ]

        responseDecoder =
            Json.map2 (\id url -> { id = id, url = url })
                (Json.field "id" Json.string)
                (Json.field "html_url" Json.string)
    in
        Http.send GistPostDone <|
            Http.post baseGistString (Http.jsonBody body) responseDecoder



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- MAIN


main : Program Never Model Msg
main =
    Navigation.program
        (\location -> NoOp)
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }
