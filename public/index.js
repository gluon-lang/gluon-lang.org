var Eval = React.createClass({
    getInitialState: function() {
        return {text: ''};
    },
    handleTextChange: function(e) {
        this.setState({text: e.target.value});
    },
    handleSubmit: function(e) {
        e.preventDefault();
        $.ajax({
          url: this.props.url,
          type: 'POST',
          dataType: 'json',
          data: this.state.text,
          success: function(data) {
            this.setState({ result: data });
          }.bind(this),
          error: function(xhr, status, err) {
            console.error(this.props.url, status, err.toString());
          }.bind(this)
        });
    },
    updateExample: function(example) {
        this.setState({text: example });
    },
    render: function() {
        return (
            <div>
                <Examples url="examples" updateExample={this.updateExample} />
                <form url="eval" onSubmit={this.handleSubmit}>
                    <textarea value={this.state.text} onChange={this.handleTextChange} name="expr" cols="100" rows="30">
                    </textarea>
                    <br/>
                    <input type="submit" value="Eval"/>
                </form>
                <h3>Result</h3>
                <pre>{this.state.result}</pre>
            </div>
        );
    }
});

var Examples = React.createClass({
    propTypes: {
        url: React.PropTypes.string.isRequired,
        updateExample: React.PropTypes.func.isRequired,
    },
    getInitialState: function() {
        return {
            options: [],
        }
    },
    componentDidMount: function() {
        $.ajax({
            url: this.props.url,
            dataType: 'json',
            success: this.successHandler
        })
    },
    successHandler: function(data) {
        for (var i = 0; i < data.length; i++) {
            var option = data[i];
            this.state.options.push(
                <option key={i} value={option.value}>{option.name}</option>
            );
        }
        // Initialize textarea with the first example
        this.props.updateExample(data[0].value);
        this.forceUpdate();
    },
    changeHandler: function(e) {
        this.props.updateExample(e.target.value);
    },
    render: function() {
        return <select onChange={this.changeHandler}>{this.state.options}</select>;
    }
});

setInterval(function() {
  ReactDOM.render(
    <Eval url="eval" />,
    document.getElementById('example')
  );
}, 500);
