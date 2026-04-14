//% IMPORTS
import android.provider.DocumentsContract;
import android.database.Cursor;
import java.io.File;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.zip.ZipOutputStream;
import java.util.zip.ZipEntry;
//% END

//% MAIN_ACTIVITY_BODY
private static final int IMPORT_DIR_REQUEST_CODE = 0x696D70;
private static volatile String importResultPath = null;
private static volatile String importMode = null;
private static volatile boolean importReady = false;

public void launchDirectoryPicker(String mode) {
	Log.i("SAPP", "launchDirectoryPicker: mode=" + mode);
	importMode = mode;
	runOnUiThread(new Runnable() {
		@Override
		public void run() {
			try {
				Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
				startActivityForResult(intent, IMPORT_DIR_REQUEST_CODE);
			} catch (Exception e) {
				Log.e("SAPP", "launchDirectoryPicker: failed", e);
			}
		}
	});
}

public static String pollImportResult() {
	if (importReady) {
		String mode = importMode;
		String path = importResultPath;
		importResultPath = null;
		importMode = null;
		importReady = false;
		if (mode != null && path != null) {
			return mode + ":" + path;
		}
		return path;
	}
	return null;
}

private void copyDocumentTree(Uri treeUri, String docId, File destDir) {
	Uri childrenUri = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, docId);
	Cursor cursor = getContentResolver().query(
		childrenUri,
		new String[]{
			DocumentsContract.Document.COLUMN_DOCUMENT_ID,
			DocumentsContract.Document.COLUMN_DISPLAY_NAME,
			DocumentsContract.Document.COLUMN_MIME_TYPE
		},
		null, null, null
	);
	if (cursor == null) return;
	try {
		while (cursor.moveToNext()) {
			String childDocId = cursor.getString(0);
			String name = cursor.getString(1);
			String mimeType = cursor.getString(2);
			if (name == null) continue;

			if (DocumentsContract.Document.MIME_TYPE_DIR.equals(mimeType)) {
				File subDir = new File(destDir, name);
				subDir.mkdirs();
				copyDocumentTree(treeUri, childDocId, subDir);
			} else {
				Uri docUri = DocumentsContract.buildDocumentUriUsingTree(treeUri, childDocId);
				File destFile = new File(destDir, name);
				try {
					InputStream input = getContentResolver().openInputStream(docUri);
					if (input != null) {
						OutputStream output = new java.io.FileOutputStream(destFile);
						byte[] buffer = new byte[8192];
						int read;
						while ((read = input.read(buffer)) != -1) {
							output.write(buffer, 0, read);
						}
						output.flush();
						output.close();
						input.close();
					}
				} catch (Exception e) {
					Log.e("SAPP", "copyDocumentTree: failed to copy " + name, e);
				}
			}
		}
	} finally {
		cursor.close();
	}
}

private void handleImportDirectoryResult(Intent data) {
	Uri treeUri = data.getData();
	if (treeUri == null) {
		Log.e("SAPP", "handleImportDirectoryResult: treeUri is null");
		return;
	}

	Log.i("SAPP", "handleImportDirectoryResult: uri=" + treeUri);
	String treeDocId = DocumentsContract.getTreeDocumentId(treeUri);

	// Derive a directory name from the URI path
	String dirName = treeDocId;
	int lastSlash = dirName.lastIndexOf('/');
	if (lastSlash >= 0) dirName = dirName.substring(lastSlash + 1);
	int lastColon = dirName.lastIndexOf(':');
	if (lastColon >= 0) dirName = dirName.substring(lastColon + 1);
	if (dirName.isEmpty()) dirName = "imported";

	File importDir = new File(getExternalFilesDir(null), "import/" + dirName);
	importDir.mkdirs();

	Log.i("SAPP", "handleImportDirectoryResult: copying to " + importDir.getAbsolutePath());
	copyDocumentTree(treeUri, treeDocId, importDir);

	importResultPath = importDir.getAbsolutePath();
	importReady = true;
	Log.i("SAPP", "handleImportDirectoryResult: done, path=" + importResultPath);
}

private static final int EXPORT_SAVE_REQUEST_CODE = 0x657870;
private static File exportZipFile = null;

public void launchExportZip(String workspacePath) {
	Log.i("SAPP", "launchExportZip: path=" + workspacePath);
	final File wsDir = new File(workspacePath);
	final String wsName = wsDir.getName();
	runOnUiThread(new Runnable() {
		@Override
		public void run() {
			try {
				exportZipFile = new File(getCacheDir(), wsName + ".zip");
				createZipFromDirectory(wsDir, exportZipFile);
				Intent intent = new Intent(Intent.ACTION_CREATE_DOCUMENT);
				intent.addCategory(Intent.CATEGORY_OPENABLE);
				intent.setType("application/zip");
				intent.putExtra(Intent.EXTRA_TITLE, wsName + ".zip");
				startActivityForResult(intent, EXPORT_SAVE_REQUEST_CODE);
			} catch (Exception e) {
				Log.e("SAPP", "launchExportZip: failed", e);
			}
		}
	});
}

private void createZipFromDirectory(File sourceDir, File zipFile) throws Exception {
	ZipOutputStream zos = new ZipOutputStream(new java.io.FileOutputStream(zipFile));
	try {
		addDirToZip(zos, sourceDir, "");
	} finally {
		zos.close();
	}
}

private void addDirToZip(ZipOutputStream zos, File dir, String prefix) throws Exception {
	File[] files = dir.listFiles();
	if (files == null) return;
	byte[] buffer = new byte[8192];
	for (File file : files) {
		String entryName = prefix.isEmpty() ? file.getName() : prefix + "/" + file.getName();
		if (file.isDirectory()) {
			addDirToZip(zos, file, entryName);
		} else {
			zos.putNextEntry(new ZipEntry(entryName));
			InputStream input = new java.io.FileInputStream(file);
			try {
				int len;
				while ((len = input.read(buffer)) > 0) {
					zos.write(buffer, 0, len);
				}
			} finally {
				input.close();
			}
			zos.closeEntry();
		}
	}
}

private void handleExportSaveResult(Intent data) {
	try {
		android.net.Uri destUri = data.getData();
		if (destUri == null || exportZipFile == null) return;
		OutputStream output = getContentResolver().openOutputStream(destUri);
		if (output == null) return;
		InputStream input = new java.io.FileInputStream(exportZipFile);
		byte[] buffer = new byte[8192];
		int read;
		while ((read = input.read(buffer)) != -1) {
			output.write(buffer, 0, read);
		}
		input.close();
		output.flush();
		output.close();
		Log.i("SAPP", "handleExportSaveResult: saved to " + destUri);
	} catch (Exception e) {
		Log.e("SAPP", "handleExportSaveResult: failed", e);
	} finally {
		if (exportZipFile != null) {
			exportZipFile.delete();
			exportZipFile = null;
		}
	}
}
//% END

//% MAIN_ACTIVITY_ON_ACTIVITY_RESULT
if (requestCode == EXPORT_SAVE_REQUEST_CODE) {
	Log.i("SAPP", "onActivityResult: EXPORT_SAVE code=" + resultCode);
	try {
		if (resultCode == Activity.RESULT_OK && data != null) {
			handleExportSaveResult(data);
		}
	} catch (Exception e) {
		Log.e("SAPP", "exportSave: failed", e);
	}
	if (exportZipFile != null) {
		exportZipFile.delete();
		exportZipFile = null;
	}
	return;
}
if (requestCode == IMPORT_DIR_REQUEST_CODE) {
	Log.i("SAPP", "onActivityResult: IMPORT_DIR code=" + resultCode);
	try {
		if (resultCode == Activity.RESULT_OK && data != null) {
			handleImportDirectoryResult(data);
		}
	} catch (Exception e) {
		Log.e("SAPP", "importDirectory: failed", e);
	}
	return;
}
//% END
