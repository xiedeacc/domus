import 'package:domus/features/backup/data/backup_settings.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  test('backup settings default to disabled with no selected albums', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final settings = await container.read(backupSettingsProvider.future);

    expect(settings.autoBackup, isFalse);
    expect(settings.backgroundBackup, isFalse);
    expect(settings.selectedAlbumIds, isEmpty);
    expect(settings.uploadedDeviceAssetIds, isEmpty);
  });

  test(
    'auto backup and selected albums persist like Immich mobile settings',
    () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      await container.read(backupSettingsProvider.notifier).setAutoBackup(true);
      await container.read(backupSettingsProvider.notifier).setSelectedAlbumIds(
        const ['isAll', 'screenshots'],
      );

      final prefs = await SharedPreferences.getInstance();
      expect(prefs.getBool(BackupSettingsNotifier.autoBackupKey), isTrue);
      expect(prefs.getStringList(BackupSettingsNotifier.selectedAlbumIdsKey), [
        'isAll',
        'screenshots',
      ]);

      final reloaded = ProviderContainer();
      addTearDown(reloaded.dispose);
      final settings = await reloaded.read(backupSettingsProvider.future);
      expect(settings.autoBackup, isTrue);
      expect(settings.selectedAlbumIds, ['isAll', 'screenshots']);
    },
  );
}
