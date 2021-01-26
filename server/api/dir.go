package api

import (
	"context"
	"log"

	"github.com/hopkings2008/yigfs/server/types"
	. "github.com/hopkings2008/yigfs/server/error"
)


func CheckAndAssignmentFileInfo(ctx context.Context, file *types.FileInfo) (err error) {
	if file.BucketName == "" || file.FileName == "" || file.Size == 0 || file.ParentIno == 0 {
		log.Printf("Some createFile required parameters are missing.")
		err = ErrYigFsMissingRequiredParams
		return
	}

	if file.Type == 0 {
		file.Type = types.COMMON_FILE
	} else {
		if file.Type != types.COMMON_FILE && file.Type != types.DIR_FILE {
			err = ErrYigFsInvalidType
			return
		}
	}

	if file.Region == "" {
		file.Region = "cn-bj-1"
	}

	if file.Perm == 0 {
		if file.Type == types.COMMON_FILE {
			file.Perm = types.FILE_PERM
		} else {
			file.Perm = types.DIR_PERM
		}
	}
	return nil
}
