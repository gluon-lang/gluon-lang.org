module Main exposing (main)

import Html exposing (Html, a, button, div, form, h2, nav, option, pre, select, text, textarea)
import Html.App
import Html.Attributes exposing (class, href, name, rows, selected)
import Html.Events exposing (onClick, onInput)
import Http
import Json.Decode as Json exposing ((:=))
import List exposing ((::))
import List.Extra as List
import Task exposing (Task)


-- MODEL


type alias Urls =
    { examples : String
    , eval : String
    }


type alias Example =
    { name : String
    , src : String
    }


type alias Model =
    { urls : Urls
    , examples : List Example
    , selectedExample : Maybe String
    , src : String
    , evalResult : String
    }


init : ( Model, Cmd Msg )
init =
    let
        model =
            { urls =
                { examples = "examples"
                , eval = "eval"
                }
            , examples = []
            , selectedExample = Nothing
            , src = ""
            , evalResult = ""
            }
    in
        ( model, getExamples model )


initExamples : List Example -> Model -> Model
initExamples examples model =
    case List.head examples of
        Just example ->
            setExample example.name { model | examples = examples }

        Nothing ->
            model


getExample : String -> Model -> Maybe String
getExample name model =
    model.examples
        |> List.find (\example -> example.name == name)
        |> Maybe.map (.src)


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
    | EvalSucceed String
    | EvalFail Http.Error
    | ExamplesSucceed (List Example)
    | ExamplesFail Http.Error
    | SelectExample String
    | EditSource String



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        EvalRequested ->
            -- TODO: Disable the eval button
            ( { model | evalResult = "Compiling..." }, postEval model )

        EvalSucceed result ->
            ( { model | evalResult = result }, Cmd.none )

        EvalFail _ ->
            ( { model | evalResult = "HTTP failure" }, Cmd.none )

        ExamplesSucceed examples ->
            ( initExamples examples model, Cmd.none )

        ExamplesFail _ ->
            ( model, Cmd.none )

        SelectExample name ->
            ( setExample name model, Cmd.none )

        EditSource src ->
            ( setSource src model, Cmd.none )



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
                (defaultOption :: List.map exampleOption model.examples)
            ]


editor : Model -> Html Msg
editor model =
    textarea [ class "editor form-control", rows 25, onInput EditSource ]
        [ text model.src ]


evalResult : Model -> Html Msg
evalResult model =
    div [ class "card" ]
        [ div [ class "card-header" ]
            [ nav [ class "nav" ]
                [ text "Result"
                , button [ class "btn btn-primary pull-xs-right", onClick EvalRequested ]
                    [ text "Eval" ]
                ]
            ]
        , div [ class "card-block" ] [ pre [] [ text model.evalResult ] ]
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
                [ a [ href "https://github.com/Marwes/gluon" ] [ text "Gluon on Github" ]
                , text " | "
                , a [ href "https://github.com/Marwes/try_gluon" ] [ text "Fork this site" ]
                ]
            ]
        ]



-- HTTP


postEval : Model -> Cmd Msg
postEval model =
    let
        evalTask =
            Http.post Json.string model.urls.eval (Http.string model.src)
    in
        Task.perform EvalFail EvalSucceed evalTask


getExamples : Model -> Cmd Msg
getExamples model =
    let
        exampleOption =
            Json.object2 (\name value -> { name = name, src = value })
                ("name" := Json.string)
                ("value" := Json.string)

        decodeExamples =
            Json.list exampleOption

        examplesTask =
            Http.get decodeExamples model.urls.examples
    in
        Task.perform ExamplesFail ExamplesSucceed examplesTask



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- MAIN


main : Program Never
main =
    Html.App.program
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }
