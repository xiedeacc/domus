import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';

class ImmichDerivativeRequest {
  const ImmichDerivativeRequest({
    this.limit = 200,
    this.repairAll = false,
    this.shardIndex = 0,
    this.shardCount = 1,
    this.resume = true,
    this.assetIds = const [],
  });

  final int limit;
  final bool repairAll;
  final int shardIndex;
  final int shardCount;
  final bool resume;
  final List<String> assetIds;

  Map<String, dynamic> toJson() => {
    'limit': limit,
    'repairAll': repairAll,
    'shardIndex': shardIndex,
    'shardCount': shardCount,
    'resume': resume,
    'assetIds': assetIds,
  };
}

class ImmichDerivativeProgress {
  const ImmichDerivativeProgress({
    required this.checked,
    required this.total,
    required this.ok,
    required this.failed,
    required this.resumed,
    required this.phase,
    required this.recentMessages,
    this.currentAssetId,
  });

  final int checked;
  final int total;
  final int ok;
  final int failed;
  final int resumed;
  final String phase;
  final String? currentAssetId;
  final List<String> recentMessages;

  factory ImmichDerivativeProgress.fromJson(Map<String, dynamic> json) {
    return ImmichDerivativeProgress(
      checked: (json['checked'] as num?)?.toInt() ?? 0,
      total: (json['total'] as num?)?.toInt() ?? 0,
      ok: (json['ok'] as num?)?.toInt() ?? 0,
      failed: (json['failed'] as num?)?.toInt() ?? 0,
      resumed: (json['resumed'] as num?)?.toInt() ?? 0,
      phase: (json['phase'] as String?) ?? 'idle',
      currentAssetId: json['currentAssetId'] as String?,
      recentMessages: [
        for (final value in (json['recentMessages'] as List? ?? const []))
          '$value',
      ],
    );
  }
}

class ImmichDerivativeSummary {
  const ImmichDerivativeSummary({
    required this.ok,
    required this.failed,
    required this.resumed,
    required this.checked,
    required this.cancelled,
  });

  final int ok;
  final int failed;
  final int resumed;
  final int checked;
  final bool cancelled;

  factory ImmichDerivativeSummary.fromJson(Map<String, dynamic> json) {
    return ImmichDerivativeSummary(
      ok: (json['ok'] as num?)?.toInt() ?? 0,
      failed: (json['failed'] as num?)?.toInt() ?? 0,
      resumed: (json['resumed'] as num?)?.toInt() ?? 0,
      checked: (json['checked'] as num?)?.toInt() ?? 0,
      cancelled: (json['cancelled'] as bool?) ?? false,
    );
  }
}

class ImmichDerivativeStatus {
  const ImmichDerivativeStatus({
    required this.running,
    required this.progress,
    this.jobId,
    this.startedAt,
    this.finishedAt,
    this.summary,
    this.error,
  });

  final bool running;
  final String? jobId;
  final String? startedAt;
  final String? finishedAt;
  final ImmichDerivativeProgress progress;
  final ImmichDerivativeSummary? summary;
  final String? error;

  factory ImmichDerivativeStatus.fromJson(Map<String, dynamic> json) {
    return ImmichDerivativeStatus(
      running: (json['running'] as bool?) ?? false,
      jobId: json['jobId'] as String?,
      startedAt: json['startedAt'] as String?,
      finishedAt: json['finishedAt'] as String?,
      progress: ImmichDerivativeProgress.fromJson(
        (json['progress'] as Map<String, dynamic>?) ?? const {},
      ),
      summary: json['summary'] is Map<String, dynamic>
          ? ImmichDerivativeSummary.fromJson(
              json['summary'] as Map<String, dynamic>,
            )
          : null,
      error: json['error'] as String?,
    );
  }
}

class ImmichDerivativeRepository {
  ImmichDerivativeRepository(this._api);

  final ApiClient _api;

  Future<ImmichDerivativeStatus> status() async {
    final response = await _api.dio.get<Map<String, dynamic>>(
      '/admin/immich-derivatives/status',
    );
    return ImmichDerivativeStatus.fromJson(response.data!);
  }

  Future<ImmichDerivativeStatus> run(ImmichDerivativeRequest request) async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/admin/immich-derivatives/run',
      data: request.toJson(),
    );
    return ImmichDerivativeStatus.fromJson(response.data!);
  }

  Future<ImmichDerivativeStatus> cancel() async {
    final response = await _api.dio.post<Map<String, dynamic>>(
      '/admin/immich-derivatives/cancel',
    );
    return ImmichDerivativeStatus.fromJson(response.data!);
  }
}

final immichDerivativeRepositoryProvider = Provider(
  (ref) => ImmichDerivativeRepository(ref.watch(apiClientProvider)),
);

final immichDerivativeStatusProvider = FutureProvider.autoDispose((ref) async {
  final repository = ref.watch(immichDerivativeRepositoryProvider);
  final status = await repository.status();
  if (status.running) {
    ref.keepAlive();
    await Future<void>.delayed(const Duration(seconds: 2));
    ref.invalidateSelf();
  }
  return status;
});
