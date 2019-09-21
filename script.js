document.addEventListener('DOMContentLoaded', function() {
    var sync = document.getElementById('sync');
    var content = document.getElementById('content');

    var addStatus = function(text, healthy) {
        var div = document.createElement('div');
        div.textContent = text;
        div.classList.add('status');
        div.classList.add(healthy ? 'healthy' : 'dead');
        content.appendChild(div);
    };

    var clear = function() {
        while (content.firstChild) {
            content.removeChild(content.firstChild);
        }
    };

    var connect = function() {
        var socket = new WebSocket('ws://' + window.location.host + '/ws/');
        socket.addEventListener('open', function(event) {
            sync.textContent = 'connected';
        });

        socket.addEventListener('message', function(event) {
            var msg = JSON.parse(event.data);
            clear();
            msg.entries.map(function(entry) {
                addStatus(entry.name, entry.healthy);
            });
        });

        socket.addEventListener('close', function(event) {
            sync.textContent = 'reconnecting...';
            setTimeout(connect, 1000);
        });
    };
    connect();
});
