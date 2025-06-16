import { useCallback } from 'react';
import { useActor } from '../ActorContextProvider';

interface IUseChronolock {
  upload: (media: ArrayBuffer) => Promise<unknown>;
}

const CHUNK_SIZE = 2 * 1024 * 1024; // 2MB

export const useChronolock = (): IUseChronolock => {
  const {
    chronolockActor: { useUpdateCall: chronolockUpdateCall },
  } = useActor();

  const {
    call: startMediaUpload,
    // data: startMediaUploadData,
    // loading: startMediaUploadLoading,
    // error: startMediaUploadError,
  } = chronolockUpdateCall({
    functionName: 'start_media_upload' as any,
  });
  const {
    call: uploadMediaChunk,
    // data: uploadMediaChunkData,
    // loading: uploadMediaChunkLoading,
    // error: uploadMediaChunkError,
  } = chronolockUpdateCall({
    functionName: 'upload_media_chunk' as any,
  });
  const {
    call: finishMediaUpload,
    // data: finishMediaUploadData,
    // loading: finishMediaUploadLoading,
    // error: finishMediaUploadError,
  } = chronolockUpdateCall({
    functionName: 'finish_media_upload' as any,
  });

  const upload = useCallback(
    async (media: ArrayBuffer) => {
      const totalChunks = Math.ceil(media.byteLength / CHUNK_SIZE);
      // 1. Start upload, get media_id
      const [mediaId] = (await startMediaUpload([totalChunks])) as [string];
      // 2. Upload each chunk
      for (let i = 0; i < totalChunks; i++) {
        const start = i * CHUNK_SIZE;
        const end = Math.min(start + CHUNK_SIZE, media.byteLength);
        const chunk = new Uint8Array(media.slice(start, end));
        await uploadMediaChunk([mediaId, i, Array.from(chunk)]);
      }
      // 3. Finish upload, get URL
      const [url] = (await finishMediaUpload([mediaId])) as [string];
      return url;
    },
    [startMediaUpload, uploadMediaChunk, finishMediaUpload],
  );

  return {
    upload,
  };
};
