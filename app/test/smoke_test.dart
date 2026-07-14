import 'package:flutter_test/flutter_test.dart';

import 'package:domus_app/models/asset.dart';

void main() {
  test('Asset.fromJson parses minimal payload', () {
    final asset = Asset.fromJson(const {
      'id': 'a1',
      'type': 'IMAGE',
      'ownerId': 'u1',
      'originalFileName': 'IMG_0001.jpg',
    });
    expect(asset.id, 'a1');
    expect(asset.isVideo, isFalse);
  });
}
