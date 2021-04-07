package api

import (
	"context"

	"github.com/hopkings2008/yigfs/server/types"
)


type YigFsLayer interface {
	ListDirFiles(ctx context.Context, dir *types.GetDirFilesReq) (listDirFilesResp []*types.GetDirFileInfo, offset uint64, err error)
	GetDirFileAttr(ctx context.Context, file *types.GetDirFileInfoReq) (resp *types.FileInfo, err error)
	GetFileAttr(ctx context.Context, file *types.GetFileInfoReq) (resp *types.FileInfo, err error)
	InitDirAndZone(ctx context.Context, rootDir *types.InitDirReq) (err error)
	GetFileLeader(ctx context.Context, leader *types.GetLeaderReq) (resp *types.GetLeaderResp, err error)
	CreateFile(ctx context.Context, file *types.CreateFileReq) (resp *types.CreateFileResp, err error)
	SetFileAttr(ctx context.Context, file *types.SetFileAttrReq) (resp *types.SetFileAttrResp, err error)
	GetFileSegmentInfo(ctx context.Context, file *types.GetSegmentReq) (resp *types.GetSegmentResp, err error)
	CreateSegmentInfo(ctx context.Context, seg *types.CreateSegmentReq, isExisted int) (err error)
	CheckSegmentLeader(ctx context.Context, segment *types.CreateSegmentReq) (isExisted int, err error)
	UpdateSegment(ctx context.Context, seg *types.CreateSegmentReq, isExisted int) (updateSegResp *types.UpdateSegResp, err error)
	GetFileSizeAndBlocksNum(ctx context.Context, seg *types.CreateSegmentReq) (size uint64, number uint32, err error)
	UpdateFileSizeAndBlocksNum(ctx context.Context, seg *types.CreateSegmentReq, allBlocksSize uint64, allBlocksNumber uint32) (err error)
}
