import * as cdk from 'aws-cdk-lib';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as origins from 'aws-cdk-lib/aws-cloudfront-origins';
import * as route53 from 'aws-cdk-lib/aws-route53';
import * as targets from 'aws-cdk-lib/aws-route53-targets';
import * as acm from 'aws-cdk-lib/aws-certificatemanager';
import * as iam from 'aws-cdk-lib/aws-iam';
import { Construct } from 'constructs';

export interface GreatShStackProps extends cdk.StackProps {
  readonly hostedZoneId: string;
}

export class GreatShStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: GreatShStackProps) {
    super(scope, id, props);

    const accountId = cdk.Stack.of(this).account;
    const domainName = 'great.sh';

    // ========================================
    // DNS + Certificate
    // ========================================

    const hostedZone = route53.HostedZone.fromHostedZoneAttributes(this, 'HostedZone', {
      hostedZoneId: props.hostedZoneId,
      zoneName: domainName,
    });

    const certificate = new acm.Certificate(this, 'Certificate', {
      domainName,
      validation: acm.CertificateValidation.fromDns(hostedZone),
    });

    // ========================================
    // S3 Buckets
    // ========================================

    const logBucket = new s3.Bucket(this, 'LogBucket', {
      bucketName: `${domainName}-logs`,
      encryption: s3.BucketEncryption.S3_MANAGED,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      removalPolicy: cdk.RemovalPolicy.RETAIN,
      autoDeleteObjects: false,
      enforceSSL: true,
      lifecycleRules: [
        {
          expiration: cdk.Duration.days(90),
          transitions: [
            {
              storageClass: s3.StorageClass.INTELLIGENT_TIERING,
              transitionAfter: cdk.Duration.days(30),
            },
          ],
        },
      ],
    });

    const distributionLogBucket = new s3.Bucket(this, 'DistributionLogBucket', {
      bucketName: `${domainName}-cloudfront-logs`,
      encryption: s3.BucketEncryption.S3_MANAGED,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      removalPolicy: cdk.RemovalPolicy.RETAIN,
      autoDeleteObjects: false,
      enforceSSL: true,
      objectOwnership: s3.ObjectOwnership.OBJECT_WRITER,
      lifecycleRules: [
        {
          expiration: cdk.Duration.days(90),
          transitions: [
            {
              storageClass: s3.StorageClass.INTELLIGENT_TIERING,
              transitionAfter: cdk.Duration.days(30),
            },
          ],
        },
      ],
    });

    const contentBucket = new s3.Bucket(this, 'ContentBucket', {
      bucketName: `${domainName}-cloudfront`,
      encryption: s3.BucketEncryption.S3_MANAGED,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      removalPolicy: cdk.RemovalPolicy.RETAIN,
      autoDeleteObjects: false,
      enforceSSL: true,
      serverAccessLogsBucket: logBucket,
      serverAccessLogsPrefix: 's3-access-logs/',
      cors: [
        {
          allowedMethods: [s3.HttpMethods.GET, s3.HttpMethods.HEAD],
          allowedOrigins: ['*'],
          allowedHeaders: ['*'],
        },
      ],
    });

    // ========================================
    // CloudFront Distribution
    // ========================================

    const distribution = new cloudfront.Distribution(this, 'Distribution', {
      defaultBehavior: {
        origin: origins.S3BucketOrigin.withOriginAccessControl(contentBucket),
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        allowedMethods: cloudfront.AllowedMethods.ALLOW_GET_HEAD_OPTIONS,
        cachedMethods: cloudfront.CachedMethods.CACHE_GET_HEAD_OPTIONS,
        compress: true,
      },
      domainNames: [domainName],
      certificate,
      priceClass: cloudfront.PriceClass.PRICE_CLASS_100,
      defaultRootObject: 'index.html',
      errorResponses: [
        {
          httpStatus: 403,
          responseHttpStatus: 200,
          responsePagePath: '/index.html',
          ttl: cdk.Duration.seconds(0),
        },
        {
          httpStatus: 404,
          responseHttpStatus: 200,
          responsePagePath: '/index.html',
          ttl: cdk.Duration.seconds(0),
        },
      ],
      enableLogging: true,
      logBucket: distributionLogBucket,
      logFilePrefix: 'cloudfront-access-logs/',
      comment: `CloudFront distribution for ${domainName}`,
    });

    // ========================================
    // Route53 A Record
    // ========================================

    new route53.ARecord(this, 'ARecord', {
      zone: hostedZone,
      recordName: domainName,
      target: route53.RecordTarget.fromAlias(
        new targets.CloudFrontTarget(distribution)
      ),
    });

    // ========================================
    // IAM: CDK Deploy Role (OIDC)
    // ========================================

    const oidcProviderArn = `arn:aws:iam::${accountId}:oidc-provider/token.actions.githubusercontent.com`;

    const cdkDeployRole = new iam.Role(this, 'CdkDeployRole', {
      roleName: 'great-sh-cdk-deploy',
      description: 'Role for GitHub Actions to deploy great.sh CDK stacks via OIDC',
      assumedBy: new iam.WebIdentityPrincipal(oidcProviderArn, {
        StringEquals: {
          'token.actions.githubusercontent.com:aud': 'sts.amazonaws.com',
        },
        StringLike: {
          'token.actions.githubusercontent.com:sub': [
            'repo:superstruct/great.sh:*',
          ],
        },
      }),
      maxSessionDuration: cdk.Duration.hours(2),
    });

    cdkDeployRole.addToPolicy(new iam.PolicyStatement({
      sid: 'CdkBootstrapRoleAssumption',
      effect: iam.Effect.ALLOW,
      actions: ['sts:AssumeRole'],
      resources: [
        `arn:aws:iam::${accountId}:role/cdk-hnb659fds-deploy-role-${accountId}-*`,
        `arn:aws:iam::${accountId}:role/cdk-hnb659fds-file-publishing-role-${accountId}-*`,
        `arn:aws:iam::${accountId}:role/cdk-hnb659fds-image-publishing-role-${accountId}-*`,
        `arn:aws:iam::${accountId}:role/cdk-hnb659fds-lookup-role-${accountId}-*`,
      ],
    }));

    cdkDeployRole.addToPolicy(new iam.PolicyStatement({
      sid: 'CloudFormationReadAccess',
      effect: iam.Effect.ALLOW,
      actions: ['cloudformation:DescribeStacks'],
      resources: [
        `arn:aws:cloudformation:us-east-1:${accountId}:stack/GreatSh*/*`,
      ],
    }));

    // ========================================
    // IAM: Content Deploy Role (OIDC)
    // ========================================

    const deployRole = new iam.Role(this, 'DeployRole', {
      roleName: 'great-sh-deploy',
      description: 'Role for GitHub Actions to deploy great.sh site content via OIDC',
      assumedBy: new iam.WebIdentityPrincipal(oidcProviderArn, {
        StringEquals: {
          'token.actions.githubusercontent.com:aud': 'sts.amazonaws.com',
        },
        StringLike: {
          'token.actions.githubusercontent.com:sub': [
            'repo:superstruct/great.sh:*',
          ],
        },
      }),
      maxSessionDuration: cdk.Duration.hours(1),
    });

    deployRole.addToPolicy(new iam.PolicyStatement({
      sid: 'S3ContentDeployment',
      effect: iam.Effect.ALLOW,
      actions: [
        's3:PutObject',
        's3:GetObject',
        's3:DeleteObject',
        's3:ListBucket',
      ],
      resources: [
        contentBucket.bucketArn,
        `${contentBucket.bucketArn}/*`,
      ],
    }));

    deployRole.addToPolicy(new iam.PolicyStatement({
      sid: 'CloudFrontInvalidation',
      effect: iam.Effect.ALLOW,
      actions: ['cloudfront:CreateInvalidation'],
      resources: [
        `arn:aws:cloudfront::${accountId}:distribution/${distribution.distributionId}`,
      ],
    }));

    // ========================================
    // Outputs
    // ========================================

    new cdk.CfnOutput(this, 'ContentBucketName', {
      value: contentBucket.bucketName,
      description: 'S3 bucket name for site content',
    });

    new cdk.CfnOutput(this, 'DistributionId', {
      value: distribution.distributionId,
      description: 'CloudFront distribution ID',
    });

    new cdk.CfnOutput(this, 'DistributionDomainName', {
      value: distribution.distributionDomainName,
      description: 'CloudFront distribution domain name',
    });

    new cdk.CfnOutput(this, 'CdkDeployRoleArn', {
      value: cdkDeployRole.roleArn,
      description: 'IAM Role ARN for GitHub Actions CDK deployment via OIDC',
    });

    new cdk.CfnOutput(this, 'DeployRoleArn', {
      value: deployRole.roleArn,
      description: 'IAM Role ARN for GitHub Actions content deployment via OIDC',
    });

    new cdk.CfnOutput(this, 'SiteUrl', {
      value: `https://${domainName}`,
      description: 'Site URL',
    });
  }
}
