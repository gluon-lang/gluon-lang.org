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
            <div className="col-lg-12">
                <div className="row">
                    <Examples url="examples" updateExample={this.updateExample} />
                </div>
                <div className="row">
                    <form className="navbar-form navbar-left" url="eval" onSubmit={this.handleSubmit}>
                        <div className="row">
                            <div className="form-group"></div>
                                <textarea className="form-control" value={this.state.text} onChange={this.handleTextChange} name="expr" cols="100" rows="30">
                                </textarea>
                            <br/>
                        </div>
                        <button type="submit" className="btn btn-primary">Eval</button>
                    </form>
                </div>
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
        return <div className="col-sm-3">
                <select className="form-control" onChange={this.changeHandler}>{this.state.options}</select>
            </div>;
    }
});

setInterval(function() {
  ReactDOM.render(
    <Eval url="eval" />,
    document.getElementById('example')
  );
}, 500);
