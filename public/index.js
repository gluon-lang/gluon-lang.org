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
    render: function() {
        return (
            <div>
                <form url="eval" onSubmit={this.handleSubmit}>
                    <textarea value={this.state.text} onChange={this.handleTextChange} name="expr" cols="100" rows="30">
                    </textarea>
                    <input type="submit" value="Eval"/>
                </form>
                <h3>Result</h3>
                <pre>{this.state.result}</pre>
            </div>
        );
    }
});

setInterval(function() {
  ReactDOM.render(
    <Eval url="eval" />,
    document.getElementById('example')
  );
}, 500);
