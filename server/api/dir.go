package api

import (
	"context"

	"github.com/hopkings2008/yigfs/server/types"
	. "github.com/hopkings2008/yigfs/server/error"
	"github.com/hopkings2008/yigfs/server/helper"
)


func CheckAndAssignmentFileInfo(ctx context.Context, file *types.CreateFileReq) (err error) {
	if file.BucketName == "" || file.FileName == "" || file.ParentIno == 0 || file.ZoneId == "" || file.Machine == "" {
		helper.Logger.Error(ctx, "Some createFile required parameters are missing.")
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

func CheckSetFileAttrParams(ctx context.Context, file *types.SetFileAttrReq) (err error) {
	if file.BucketName == "" || file.File.Ino == 0 {
		helper.Logger.Error(ctx, "Some SetFileAttr required parameters are missing.")
		err = ErrYigFsMissingRequiredParams
		return
	}

	if file.Region == "" {
		file.Region = "cn-bj-1"
	}

	return nil
}

