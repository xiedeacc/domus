import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';

class Partner {
  const Partner({
    required this.id,
    required this.email,
    required this.name,
    required this.inTimeline,
  });

  final String id;
  final String email;
  final String name;
  final bool inTimeline;

  factory Partner.fromJson(Map<String, dynamic> json) => Partner(
    id: json['id'] as String,
    email: json['email'] as String? ?? '',
    name: json['name'] as String? ?? '',
    inTimeline: (json['inTimeline'] as bool?) ?? false,
  );
}

class PartnerRepository {
  PartnerRepository(this._api);

  final ApiClient _api;

  Future<List<Partner>> list() async {
    final response = await _api.dio.get<List<dynamic>>(
      '/partners',
      queryParameters: {'direction': 'shared-by'},
    );
    return [
      for (final item in response.data!)
        Partner.fromJson(item as Map<String, dynamic>),
    ];
  }

  Future<void> create(String userId) async {
    await _api.dio.post<void>('/partners', data: {'sharedWithId': userId});
  }

  Future<void> remove(String userId) async {
    await _api.dio.delete<void>('/partners/$userId');
  }
}

final partnerRepositoryProvider = Provider(
  (ref) => PartnerRepository(ref.watch(apiClientProvider)),
);

final partnersProvider = FutureProvider<List<Partner>>(
  (ref) => ref.watch(partnerRepositoryProvider).list(),
);
