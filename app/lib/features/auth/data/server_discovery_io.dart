import 'dart:async';
import 'dart:convert';
import 'dart:io';

class ServerDiscovery {
  Future<String?> discover({
    Duration timeout = const Duration(seconds: 2),
  }) async {
    final socket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, 0);
    socket.broadcastEnabled = true;
    final completer = Completer<String?>();
    late final StreamSubscription<RawSocketEvent> sub;
    sub = socket.listen((event) {
      if (event != RawSocketEvent.read) return;
      final datagram = socket.receive();
      if (datagram == null) return;
      try {
        final body =
            jsonDecode(utf8.decode(datagram.data)) as Map<String, dynamic>;
        final url = body['url'] as String?;
        if (url != null && !completer.isCompleted) {
          completer.complete(url);
        }
      } catch (_) {
        // Ignore unrelated UDP traffic.
      }
    });
    socket.send(
      utf8.encode('DOMUS_DISCOVER_V1'),
      InternetAddress('255.255.255.255'),
      43004,
    );
    socket.send(
      utf8.encode('DOMUS_DISCOVER_V1'),
      InternetAddress('255.255.255.255'),
      2283,
    );
    final result = await completer.future.timeout(
      timeout,
      onTimeout: () => null,
    );
    await sub.cancel();
    socket.close();
    return result;
  }
}
